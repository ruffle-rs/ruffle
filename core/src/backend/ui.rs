pub trait UiBackend {
    fn is_fullscreen(&self) -> bool;
    fn message(&self, message: &str);
}

/// UiBackend that does mostly nothing
pub struct NullUiBackend {}

impl NullUiBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiBackend for NullUiBackend {
    fn is_fullscreen(&self) -> bool {
        false
    }
    fn message(&self, _message: &str) {}
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}
