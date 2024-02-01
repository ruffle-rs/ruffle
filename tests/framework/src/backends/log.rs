use ruffle_core::backend::log::LogBackend;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TestLogBackend {
    trace_output: Rc<RefCell<String>>,
}

impl Default for TestLogBackend {
    fn default() -> Self {
        Self {
            trace_output: Rc::new(RefCell::new(String::new())),
        }
    }
}

impl TestLogBackend {
    pub fn trace_output(&self) -> String {
        self.trace_output.take()
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push_str(message);
        self.trace_output.borrow_mut().push('\n');
    }
}
