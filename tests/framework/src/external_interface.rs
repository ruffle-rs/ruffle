use ruffle_core::backend::log::LogBackend;
use ruffle_core::context::UpdateContext;
use ruffle_core::external::{ExternalInterfaceProvider, Value};

use crate::backends::TestLogBackend;

pub struct TestExternalInterfaceProvider {
    log: TestLogBackend,
}

impl TestExternalInterfaceProvider {
    pub fn new(log: TestLogBackend) -> Self {
        TestExternalInterfaceProvider { log }
    }
}

impl ExternalInterfaceProvider for TestExternalInterfaceProvider {
    fn call_method(&self, _context: &mut UpdateContext<'_>, name: &str, _args: &[Value]) -> Value {
        self.log
            .avm_trace(&format!("ExternalInterfaceProvider::call_method {name}"));
        Value::Undefined
    }

    fn on_callback_available(&self, name: &str) {
        self.log.avm_trace(&format!(
            "ExternalInterfaceProvider::on_callback_available {name}"
        ));
    }

    fn get_id(&self) -> Option<String> {
        None
    }
}
