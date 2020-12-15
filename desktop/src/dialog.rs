use tinyfiledialogs::{
    message_box_ok, message_box_ok_cancel, message_box_yes_no, MessageBoxIcon, OkCancel, YesNo,
};

use ruffle_core::backend::dialog::DialogBackend;

pub struct DesktopDialogBackend {}

impl DesktopDialogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl DialogBackend for DesktopDialogBackend {
    fn message(&self, title: &str, message: &str) {
        message_box_ok(title, message, MessageBoxIcon::Info)
    }

    fn yes_no(&self, message: &str) -> bool {
        match message_box_yes_no(
            "Confirm Dialog",
            message,
            MessageBoxIcon::Question,
            YesNo::Yes,
        ) {
            YesNo::Yes => true,
            YesNo::No => false,
        }
    }

    fn ok_cancel(&self, message: &str) -> bool {
        match message_box_ok_cancel(
            "Confirm Dialog",
            message,
            MessageBoxIcon::Question,
            OkCancel::Ok,
        ) {
            OkCancel::Ok => true,
            OkCancel::Cancel => false,
        }
    }
}

impl Default for DesktopDialogBackend {
    fn default() -> Self {
        DesktopDialogBackend::new()
    }
}
