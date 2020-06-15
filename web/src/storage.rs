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

//TODO: scope to current url
impl StorageBackend for LocalStorageBackend {
    fn get_string(&self, name: &str) -> Option<String> {
        self.storage.get(name).unwrap()
    }

    fn put_string(&mut self, name: &str, value: String) -> bool {
        self.storage.set(name, &value).is_ok()
    }

    fn remove_key(&mut self, name: &str) {
        let _ = self.storage.delete(name);
    }
}
