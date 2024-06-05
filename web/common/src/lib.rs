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

impl From<JsError> for JsValue {
    fn from(error: JsError) -> Self {
        error.value
    }
}

pub trait JsResult<T> {
    /// Converts a `JsValue` into a standard `Error`.
    fn warn_on_error(&self);
    fn into_js_result(self) -> Result<T, JsError>;
}

impl<T> JsResult<T> for Result<T, JsValue> {
    #[inline]
    fn warn_on_error(&self) {
        if let Err(value) = self {
            tracing::warn!("Unexpected JavaScript error: {:?}", value);
        }
    }

    #[inline]
    fn into_js_result(self) -> Result<T, JsError> {
        self.map_err(|value| JsError { value })
    }
}
