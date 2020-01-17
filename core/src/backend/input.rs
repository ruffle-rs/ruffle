use crate::events::KeyCode;

pub trait InputBackend {
    fn is_key_down(&self, key: KeyCode) -> bool;

    fn get_last_key_code(&self) -> KeyCode;

    fn mouse_visible(&self) -> bool;

    fn hide_mouse(&mut self);

    fn show_mouse(&mut self);
}

/// Input backend that does nothing
pub struct NullInputBackend {}

impl NullInputBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputBackend for NullInputBackend {
    fn is_key_down(&self, _key: KeyCode) -> bool {
        false
    }

    fn get_last_key_code(&self) -> KeyCode {
        KeyCode::Unknown
    }

    fn mouse_visible(&self) -> bool {
        true
    }

    fn hide_mouse(&mut self) {}

    fn show_mouse(&mut self) {}
}

impl Default for NullInputBackend {
    fn default() -> Self {
        NullInputBackend::new()
    }
}
