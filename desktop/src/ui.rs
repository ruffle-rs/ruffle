use tinyfiledialogs::{message_box_ok, MessageBoxIcon};

use ruffle_core::backend::ui::UiBackend;

pub struct DesktopUiBackend {}

impl DesktopUiBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiBackend for DesktopUiBackend {
    fn message(&self, message: &str) {
        message_box_ok("Ruffle", message, MessageBoxIcon::Info)
    }
}

impl Default for DesktopUiBackend {
    fn default() -> Self {
        DesktopUiBackend::new()
    }
}
