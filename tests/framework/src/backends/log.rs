use ruffle_core::backend::log::LogBackend;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TestLogBackend {
    trace_output: Rc<RefCell<String>>,
    log_warnings: bool,
}

impl TestLogBackend {
    pub fn new(log_warnings: bool) -> Self {
        Self {
            trace_output: Rc::new(RefCell::new(String::new())),
            log_warnings,
        }
    }

    pub fn trace_output(&self) -> String {
        self.trace_output.take()
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push_str(message);
        self.trace_output.borrow_mut().push('\n');
    }

    fn avm_warning(&self, message: &str) {
        if self.log_warnings {
            // Match the format used by Flash Player
            self.trace_output.borrow_mut().push_str("Warning: ");
            self.trace_output.borrow_mut().push_str(message);
            self.trace_output.borrow_mut().push('\n');
        }
    }
}
