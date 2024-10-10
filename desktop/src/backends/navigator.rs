use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use ruffle_frontend_utils::backends::navigator::NavigatorInterface;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use url::Url;
use winit::event_loop::EventLoopProxy;

use crate::cli::{FilesystemAccessMode, OpenUrlMode};
use crate::custom_event::RuffleEvent;
use crate::gui::dialogs::filesystem_access_dialog::{
    FilesystemAccessDialogConfiguration, FilesystemAccessDialogResult,
};
use crate::gui::dialogs::network_access_dialog::{
    NetworkAccessDialogConfiguration, NetworkAccessDialogResult,
};
use crate::gui::DialogDescriptor;
use crate::preferences::GlobalPreferences;
use crate::util::open_url;

// TODO Make this more generic, maybe a manager?
#[derive(Clone)]
pub struct PathAllowList {
    allowed_path_prefixes: Arc<Mutex<Vec<PathBuf>>>,
}

impl PathAllowList {
    fn new(movie_path: Option<PathBuf>) -> Self {
        let mut allowed_path_prefixes = Vec::new();
        if let Some(movie_path) = movie_path {
            if let Some(parent) = movie_path.parent() {
                // TODO Be smarter here, we do not necessarily want to allow
                //   access to the SWF's dir, but we also do not want to present
                //   the dialog to the user too often.
                allowed_path_prefixes.push(parent.to_path_buf());
            }
            allowed_path_prefixes.push(movie_path);
        }
        Self {
            allowed_path_prefixes: Arc::new(Mutex::new(allowed_path_prefixes)),
        }
    }

    /// Checks whether the user already allowed access to the file.
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        for path_prefix in self
            .allowed_path_prefixes
            .lock()
            .expect("Non-poisoned lock")
            .as_slice()
        {
            let Ok(path_prefix) = path_prefix.canonicalize() else {
                continue;
            };
            let Ok(path) = path.canonicalize() else {
                continue;
            };
            if path.starts_with(path_prefix) {
                return true;
            }
        }
        false
    }

    pub fn add_allowed_path_prefix(&self, path_prefix: PathBuf) {
        self.allowed_path_prefixes
            .lock()
            .expect("Non-poisoned lock")
            .push(path_prefix);
    }
}

#[derive(Clone)]
pub struct DesktopNavigatorInterface {
    preferences: GlobalPreferences,

    // Arc + Mutex due to macOS
    event_loop: Arc<Mutex<EventLoopProxy<RuffleEvent>>>,

    filesystem_access_mode: FilesystemAccessMode,

    allow_list: PathAllowList,
}

impl DesktopNavigatorInterface {
    pub fn new(
        preferences: GlobalPreferences,
        event_loop: EventLoopProxy<RuffleEvent>,
        movie_path: Option<PathBuf>,
        filesystem_access_mode: FilesystemAccessMode,
    ) -> Self {
        Self {
            preferences,
            event_loop: Arc::new(Mutex::new(event_loop)),
            allow_list: PathAllowList::new(movie_path),
            filesystem_access_mode,
        }
    }

    async fn ask_for_filesystem_access(&self, path: &Path) -> bool {
        let (notifier, receiver) = oneshot::channel();
        let _ = self
            .event_loop
            .lock()
            .expect("Non-poisoned event loop")
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::FilesystemAccess(
                FilesystemAccessDialogConfiguration::new(
                    notifier,
                    self.allow_list.clone(),
                    path.to_path_buf(),
                ),
            )));

        receiver.await == Ok(FilesystemAccessDialogResult::Allow)
    }
}

impl NavigatorInterface for DesktopNavigatorInterface {
    fn navigate_to_website(&self, url: Url) {
        let open_url_mode = self.preferences.open_url_mode();
        if open_url_mode == OpenUrlMode::Deny {
            tracing::warn!("SWF tried to open a website, but opening a website is not allowed");
            return;
        }

        if open_url_mode == OpenUrlMode::Allow {
            open_url(&url);
            return;
        }

        let _ = self
            .event_loop
            .lock()
            .expect("Non-poisoned event loop")
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::OpenUrl(url)));
    }

    async fn open_file(&self, path: &Path) -> io::Result<File> {
        let path = &path.canonicalize()?;

        let allow = if self.allow_list.is_path_allowed(path) {
            true
        } else {
            match self.filesystem_access_mode {
                FilesystemAccessMode::Allow => true,
                FilesystemAccessMode::Deny => false,
                FilesystemAccessMode::Ask => self.ask_for_filesystem_access(path).await,
            }
        };

        if !allow {
            return Err(ErrorKind::PermissionDenied.into());
        }

        File::open(path).or_else(|e| {
            if cfg!(feature = "sandbox") {
                use rfd::FileDialog;
                let parent_path = path.parent().unwrap_or(path);

                if e.kind() == ErrorKind::PermissionDenied {
                    let attempt_sandbox_open = MessageDialog::new()
                        .set_level(MessageLevel::Warning)
                        .set_description(format!("The current movie is attempting to read files stored in {parent_path:?}.\n\nTo allow it to do so, click Yes, and then Open to grant read access to that directory.\n\nOtherwise, click No to deny access."))
                        .set_buttons(MessageButtons::YesNo)
                        .show() == MessageDialogResult::Yes;

                    if attempt_sandbox_open {
                        FileDialog::new().set_directory(parent_path).pick_folder();

                        return File::open(path);
                    }
                }
            }

            Err(e)
        })
    }

    async fn confirm_socket(&self, host: &str, port: u16) -> bool {
        let (notifier, receiver) = oneshot::channel();
        let _ = self
            .event_loop
            .lock()
            .expect("Non-poisoned event loop")
            .send_event(RuffleEvent::OpenDialog(DialogDescriptor::NetworkAccess(
                NetworkAccessDialogConfiguration::new(notifier, host, port),
            )));
        let result = receiver.await;
        result == Ok(NetworkAccessDialogResult::Allow)
    }
}
