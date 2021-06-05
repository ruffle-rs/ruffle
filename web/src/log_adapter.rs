use js_sys::Function;
use ruffle_core::backend::log::LogBackend;
use std::cell::RefCell;
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};

pub struct WebLogBackend {
    trace_observer: Arc<RefCell<JsValue>>,
}

impl WebLogBackend {
    pub fn new(trace_observer: Arc<RefCell<JsValue>>) -> Self {
        Self { trace_observer }
    }
}

impl LogBackend for WebLogBackend {
    fn avm_trace(&self, message: &str) {
        log::info!(target: "avm_trace", "{}", message);
        if let Some(function) = self.trace_observer.borrow().dyn_ref::<Function>() {
            let _ = function.call1(function, &JsValue::from_str(message));
        }
    }
}
