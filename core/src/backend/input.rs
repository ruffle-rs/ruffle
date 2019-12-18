use crate::events::KeyCode;

pub trait InputBackend {
    fn is_key_down(&self, key: KeyCode) -> bool;
}

/// Input backend that does nothing
pub struct NullInputBackend {}

impl NullInputBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputBackend for NullInputBackend {
    fn is_key_down(&self, _key: KeyCode) -> bool {
        false
    }
}

impl Default for NullInputBackend {
    fn default() -> Self {
        NullInputBackend::new()
    }
}
