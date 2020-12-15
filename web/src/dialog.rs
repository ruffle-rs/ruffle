use wasm_bindgen::prelude::*;

use ruffle_core::backend::dialog::DialogBackend;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    fn confirm(s: &str) -> bool;
}

impl WebDialogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct WebDialogBackend {}

impl DialogBackend for WebDialogBackend {
    fn message(&self, message: &str) {
        alert(message);
    }

    fn yes_no(&self, message: &str) -> bool {
        confirm(message)
    }

    fn ok_cancel(&self, message: &str) -> bool {
        confirm(message)
    }
}

impl Default for WebDialogBackend {
    fn default() -> Self {
        WebDialogBackend::new()
    }
}
