use std::collections::HashMap;

pub trait StorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>>;

    fn put(&mut self, name: &str, value: &[u8]) -> bool;

    #[inline]
    fn get_size(&self, name: &str) -> Option<usize> {
        self.get(name).map(|x| x.len())
    }

    fn remove_key(&mut self, name: &str);
}

#[derive(Default)]
pub struct MemoryStorageBackend {
    map: HashMap<String, Vec<u8>>,
}

impl MemoryStorageBackend {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageBackend for MemoryStorageBackend {
    #[inline]
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        self.map.get(name).cloned()
    }

    #[inline]
    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        self.map.insert(name.into(), value.to_vec());
        true
    }

    #[inline]
    fn remove_key(&mut self, name: &str) {
        self.map.remove(name);
    }
}
