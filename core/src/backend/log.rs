pub trait LogBackend {
    fn avm_trace(&self, message: &str);

    fn avm_warning(&self, message: &str);
}

/// Logging backend that just reroutes traces to the log crate
pub struct NullLogBackend {}

impl NullLogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl LogBackend for NullLogBackend {
    fn avm_trace(&self, message: &str) {
        tracing::info!(target: "avm_trace", "{}", message);
    }

    fn avm_warning(&self, message: &str) {
        tracing::info!(target: "avm_warning", "{}", message);
    }
}

impl Default for NullLogBackend {
    fn default() -> Self {
        NullLogBackend::new()
    }
}
