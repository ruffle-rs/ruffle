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

// impl SharedObjectBackend for MemoryBackend {
// }
//
// // In the desktop player
// struct DiskBackend {}
//
// impl SharedObjectBackend for DiskBackend {
//     pub fn get_obj_json(name: String) -> String {
//         // read from file
//     }
//     pub fn set_obj_json(name: String, value: String) {
//         // write to file
//         //TODO: if slow maybe keep and internal cache, also consider async io for fast flushing
//     }
//
//     pub fn clear() {
//         // Delete the file, modifying the avm objects from any of these will be done in shared_object
//     }
// }
//
// // for web player
// struct LocalStorageBackend {}
//
// //TODO: check the issue about this, need to prefix with url of site somehow to avoid collisions
