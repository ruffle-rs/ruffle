//! Utility functions for the web backend.
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct JsError {
    value: JsValue,
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "JS exception: {:?}", self.value)
    }
}

impl std::error::Error for JsError {}

pub trait JsResult<T> {
    /// Converts a `JsValue` into a standard `Error`.
    fn warn_on_error(&self);
    fn into_js_result(self) -> Result<T, JsError>;
}

impl<T> JsResult<T> for Result<T, JsValue> {
    #[inline]
    fn warn_on_error(&self) {
        if let Err(value) = self {
            log::warn!("Unexpected JavaScript error: {:?}", value);
        }
    }

    #[inline]
    fn into_js_result(self) -> Result<T, JsError> {
        self.map_err(|value| JsError { value })
    }
}

/// Very bad way to guess if we're running on a tablet/mobile.
pub fn is_mobile_or_tablet() -> bool {
    if let Some(window) = web_sys::window() {
        if let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from("orientation")) {
            return !val.is_undefined();
        }
    }

    false
}
