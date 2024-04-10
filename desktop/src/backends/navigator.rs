use rfd::{AsyncMessageDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use ruffle_frontend_utils::backends::navigator::NavigatorInterface;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use url::Url;

#[derive(Clone)]
pub struct RfdNavigatorInterface;

impl NavigatorInterface for RfdNavigatorInterface {
    fn confirm_website_navigation(&self, url: &Url) -> bool {
        let message = format!("The SWF file wants to open the website {}", url);
        // TODO: Add a checkbox with a GUI toolkit
        MessageDialog::new()
            .set_title("Open website?")
            .set_level(MessageLevel::Info)
            .set_description(message)
            .set_buttons(MessageButtons::OkCancel)
            .show()
            == MessageDialogResult::Ok
    }

    fn open_file(&self, path: &Path) -> io::Result<File> {
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
        AsyncMessageDialog::new().set_level(MessageLevel::Warning).set_description(format!("The current movie is attempting to connect to {:?} (port {}).\n\nTo allow it to do so, click Yes to grant network access to that host.\n\nOtherwise, click No to deny access.", host, port)).set_buttons(MessageButtons::YesNo)
            .show()
            .await == MessageDialogResult::Yes
    }
}
