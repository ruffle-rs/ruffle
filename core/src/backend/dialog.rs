pub trait DialogBackend {
    fn message(&self, message: &str);
    fn yes_no(&self, message: &str) -> bool;
    fn ok_cancel(&self, message: &str) -> bool;
}

/// DialogBackend that does mostly nothing
///
/// For tests, this backend will return true for question dialogs
/// as positive responses are usually less disruptive.

pub struct NullDialogBackend {}

impl NullDialogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl DialogBackend for NullDialogBackend {
    fn message(&self, _message: &str) {}

    fn yes_no(&self, _message: &str) -> bool {
        true
    }

    fn ok_cancel(&self, _message: &str) -> bool {
        true
    }
}

impl Default for NullDialogBackend {
    fn default() -> Self {
        NullDialogBackend::new()
    }
}
