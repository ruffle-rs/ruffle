use downcast_rs::Downcast;
use std::collections::HashMap;

pub trait StorageBackend: Downcast {
    fn get(&self, name: &str) -> Option<Vec<u8>>;

    fn put(&mut self, name: &str, value: &[u8]) -> bool;

    fn get_size(&self, name: &str) -> Option<usize> {
        self.get(name).map(|x| x.len())
    }

    fn remove_key(&mut self, name: &str);
}
impl_downcast!(StorageBackend);

#[derive(Default)]
pub struct MemoryStorageBackend {
    map: HashMap<String, Vec<u8>>,
}

impl StorageBackend for MemoryStorageBackend {
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
