pub trait UiBackend {
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
    fn message(&self, _message: &str) {}
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}
