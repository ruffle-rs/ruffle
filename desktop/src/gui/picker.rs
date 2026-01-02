use super::{LocalizableText, text};
use crate::custom_event::RuffleEvent;
use crate::gui::DialogDescriptor;
use crate::gui::dialogs::pick_path_dialog::{PickPathDialogConfiguration, PickPathDialogResult};
use crate::preferences::GlobalPreferences;
use rfd::AsyncFileDialog;
use ruffle_frontend_utils::bundle::Bundle;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::oneshot;
use walkdir::{DirEntry, WalkDir};
use winit::{event_loop::EventLoopProxy, window::Window};

#[derive(Clone)]
pub struct FilePicker {
    data: Arc<FilePickerData>,
}

struct FilePickerData {
    event_loop: EventLoopProxy<RuffleEvent>,
    parent: Weak<Window>,
    picking: AtomicBool,
    preferences: GlobalPreferences,
}

impl FilePicker {
    pub fn new(
        parent: Weak<Window>,
        preferences: GlobalPreferences,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        Self {
            data: Arc::new(FilePickerData {
                event_loop,
                parent,
                picking: AtomicBool::new(false),
                preferences,
            }),
        }
    }

    pub fn show_dialog<F, O>(&self, mut dialog: AsyncFileDialog, f: F) -> Option<O>
    where
        F: FnOnce(AsyncFileDialog) -> O,
    {
        if let Some(parent) = self.data.parent.upgrade() {
            dialog = dialog.set_parent(&parent);
        }

        if self.data.picking.swap(true, Ordering::SeqCst) {
            // Already picking
            return None;
        }
        let result = f(dialog);
        self.data.picking.store(false, Ordering::SeqCst);
        Some(result)
    }

    pub async fn pick_ruffle_file(&self, dir: Option<PathBuf>) -> Option<PathBuf> {
        let locale = &self.data.preferences.language();
        let mut dialog = AsyncFileDialog::new()
            .add_filter(
                text(locale, "file-picker-filter-supported"),
                &["swf", "spl", "ruf"],
            )
            .add_filter(text(locale, "file-picker-filter-swf"), &["swf"])
            .add_filter(text(locale, "file-picker-filter-spl"), &["spl"])
            .add_filter(text(locale, "file-picker-filter-ruf"), &["ruf"])
            .add_filter(text(locale, "file-picker-filter-all"), &["*"])
            .set_title(text(locale, "file-picker-title-open-file"));

        if let Some(dir) = dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(result) = self.show_dialog(dialog, |d| d.pick_file()) {
            result.await.map(|h| h.into())
        } else {
            None
        }
    }

    pub async fn pick_ruffle_directory(&self, dir: Option<PathBuf>) -> Option<PathBuf> {
        let locale = &self.data.preferences.language();
        let mut dialog =
            AsyncFileDialog::new().set_title(text(locale, "file-picker-title-open-directory"));

        if let Some(ref dir) = dir {
            dialog = dialog.set_directory(dir);
        }

        Some(self.show_dialog(dialog, |d| d.pick_folder())?.await?.into())
    }

    /// Pick a directory and content to play.
    ///
    /// This can return:
    ///  * a Ruffle bundle as a directory (`/dir/bundle_dir`, `/dir/bundle_dir`),
    ///  * a multi-asset SWF movie (`/dir/root_dir`, `/dir/root_dir/root_movie.swf`).
    pub async fn pick_ruffle_directory_and_content(
        &self,
        dir: Option<PathBuf>,
    ) -> Option<(PathBuf, PathBuf)> {
        let directory = self.pick_ruffle_directory(dir).await?;

        // Try opening a bundle...
        match Bundle::from_path(&directory) {
            Ok(_) => {
                // It's a bundle.
                return Some((directory.clone(), directory));
            }
            Err(err) => {
                tracing::info!("The opened directory is not a bundle: {err}");
            }
        }

        // If there's only one SWF in the directory, treat is as the root...
        let mut files: Vec<DirEntry> = WalkDir::new(&directory)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.file_name().as_encoded_bytes().ends_with(b".swf"))
            .take(2)
            .collect();
        if files.len() == 1 {
            let file = files.remove(0);
            tracing::info!(
                "There's only one .swf file in the directory, treating it as the root: {}",
                file.path().to_string_lossy()
            );
            return Some((directory, file.path().to_path_buf()));
        }

        // If there are multiple SWFs, show a dialog for picking one...
        let (notifier, receiver) = oneshot::channel();
        let _ =
            self.data
                .event_loop
                .send_event(RuffleEvent::OpenDialog(DialogDescriptor::PickPath(
                    PickPathDialogConfiguration::new(
                        notifier,
                        directory.clone(),
                        LocalizableText::LocalizedText("file-picker-pick-root-movie-title"),
                        Some(LocalizableText::LocalizedText(
                            "file-picker-pick-root-movie-message",
                        )),
                        LocalizableText::LocalizedText("file-picker-pick-root-movie-pick"),
                        Some("swf"),
                    ),
                )));
        let PickPathDialogResult::PickPath(root_movie) = receiver.await.ok()? else {
            return None;
        };

        if !root_movie.starts_with(&directory) {
            tracing::error!(
                "Picked a file outside of the directory: {}, {}",
                root_movie.to_string_lossy(),
                directory.to_string_lossy()
            );
            return None;
        }

        Some((directory, root_movie))
    }
}
