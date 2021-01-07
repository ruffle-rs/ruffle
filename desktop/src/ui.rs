use ruffle_core::backend::ui::UiBackend;
use std::rc::Rc;
use tinyfiledialogs::{message_box_ok, MessageBoxIcon};
use winit::window::Window;

pub struct DesktopUiBackend {
    window: Rc<Window>,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>) -> Self {
        Self { window }
    }
}

impl UiBackend for DesktopUiBackend {
    fn is_fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    fn message(&self, message: &str) {
        message_box_ok("Ruffle", message, MessageBoxIcon::Info)
    }
}
