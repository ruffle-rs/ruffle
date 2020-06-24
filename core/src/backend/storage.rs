use downcast_rs::Downcast;
use std::collections::HashMap;

pub trait StorageBackend: Downcast {
    fn get_string(&self, name: &str) -> Option<String>;

    fn put_string(&mut self, name: &str, value: String) -> bool;

    fn get_size(&self, name: &str) -> Option<usize> {
        self.get_string(name).map(|x| x.as_bytes().len())
    }

    fn remove_key(&mut self, name: &str);
}
impl_downcast!(StorageBackend);

pub struct MemoryStorageBackend {
    pub map: HashMap<String, String>,
}

impl Default for MemoryStorageBackend {
    fn default() -> Self {
        MemoryStorageBackend {
            map: HashMap::new(),
        }
    }
}

impl StorageBackend for MemoryStorageBackend {
    fn get_string(&self, name: &str) -> Option<String> {
        self.map.get(name).cloned()
    }

    fn put_string(&mut self, name: &str, value: String) -> bool {
        self.map.insert(name.into(), value);
        true
    }

    fn remove_key(&mut self, name: &str) {
        self.map.remove(name);
    }
}
