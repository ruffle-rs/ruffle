use ruffle_core::backend::log::LogBackend;

pub struct StdoutLogBackend {}

impl StdoutLogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl LogBackend for StdoutLogBackend {
    fn avm_trace(&self, message: &str) {
        log::info!(target: "avm_trace", "{}", message);
        println!("{}", message);
    }
}
