use ruffle_core::backend::storage::StorageBackend;
use std::collections::HashMap;

pub struct TestStorageBackend {
    map: HashMap<String, Vec<u8>>,
}

impl TestStorageBackend {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Exposes the inner data so we can use them in tests.
    pub fn get_stored_data(&self) -> &HashMap<String, Vec<u8>> {
        &self.map
    }
}

impl StorageBackend for TestStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        self.map.get(name).cloned()
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        self.map.insert(name.into(), value.to_vec());
        true
    }

    fn remove_key(&mut self, name: &str) {
        self.map.remove(name);
    }
}
