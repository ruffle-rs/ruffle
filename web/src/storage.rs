use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ruffle_core::backend::storage::StorageBackend;
use web_sys::Storage;

pub struct LocalStorageBackend {
    storage: Storage,
}

impl LocalStorageBackend {
    pub(crate) fn new(storage: Storage) -> Self {
        LocalStorageBackend { storage }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        if let Ok(Some(data)) = self.storage.get(name) {
            if let Ok(data) = BASE64_STANDARD.decode(data) {
                return Some(data);
            }
        }

        None
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        self.storage
            .set(name, &BASE64_STANDARD.encode(value))
            .is_ok()
    }

    fn remove_key(&mut self, name: &str) {
        let _ = self.storage.delete(name);
    }
}
