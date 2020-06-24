use ruffle_core::backend::storage::StorageBackend;
use web_sys::Storage;

pub struct LocalStorageBackend {
    storage: Storage,
    prefix: String,
}

impl LocalStorageBackend {
    pub(crate) fn new(storage: Storage, prefix: String) -> Self {
        LocalStorageBackend { storage, prefix }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn get_string(&self, name: &str) -> Option<String> {
        self.storage
            .get(&format!("{}-{}", self.prefix, name))
            .unwrap_or_default()
    }

    fn put_string(&mut self, name: &str, value: String) -> bool {
        self.storage
            .set(&format!("{}-{}", self.prefix, name), &value)
            .is_ok()
    }

    fn remove_key(&mut self, name: &str) {
        let _ = self.storage.delete(&format!("{}-{}", self.prefix, name));
    }
}
