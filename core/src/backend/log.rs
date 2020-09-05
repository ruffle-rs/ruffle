pub trait LogBackend {
    fn avm_trace(&self, message: &str);
}

/// Logging backend that does nothing.
pub struct NullLogBackend {}

impl NullLogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl LogBackend for NullLogBackend {
    fn avm_trace(&self, _message: &str) {}
}

impl Default for NullLogBackend {
    fn default() -> Self {
        NullLogBackend::new()
    }
}
