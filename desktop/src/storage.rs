use ruffle_core::backend::storage::StorageBackend;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub struct DiskStorageBackend {
    base_path: PathBuf,
    shared_objects_path: PathBuf,
}

impl DiskStorageBackend {
    pub fn new() -> Self {
        let base_path = dirs::data_local_dir().unwrap().join("ruffle");
        let shared_objects_path = base_path.join("SharedObjects");

        // Create a base dir if one doesn't exist yet
        if !shared_objects_path.exists() {
            log::info!("Creating storage dir");
            if let Err(r) = fs::create_dir_all(&base_path) {
                log::warn!("Unable to create storage dir {}", r);
            }
        }

        DiskStorageBackend {
            base_path,
            shared_objects_path,
        }
    }

    /// Verifies that the path contains no `..` components to prevent accessing files outside of the Ruffle directory.
    fn is_path_allowed(path: &Path) -> bool {
        path.components().all(|c| c != Component::ParentDir)
    }

    fn get_shared_object_path(&self, name: &str) -> PathBuf {
        self.shared_objects_path.join(format!("{name}.sol"))
    }

    fn get_back_compat_shared_object_path(&self, name: &str) -> PathBuf {
        // Backwards compatibility with pre-05/09/2021:
        // Search for data in old location, without .sol extension and # prefix.
        // Remove this code eventually.
        self.base_path.join(name.replacen("/#", "/", 1))
    }
}

impl StorageBackend for DiskStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.get_shared_object_path(name);
        if !Self::is_path_allowed(&path) {
            return None;
        }
        match std::fs::read(path) {
            Ok(data) => Some(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let path = self.get_back_compat_shared_object_path(name);
                match std::fs::read(path) {
                    Ok(data) => Some(data),
                    Err(e) => {
                        log::warn!("Unable to read file {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Unable to read file {:?}", e);
                None
            }
        }
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        let path = self.get_shared_object_path(name);
        if !Self::is_path_allowed(&path) {
            return false;
        }
        if let Some(parent_dir) = path.parent() {
            if !parent_dir.exists() {
                if let Err(r) = fs::create_dir_all(&parent_dir) {
                    log::warn!("Unable to create storage dir {}", r);
                    return false;
                }
            }
        }

        match File::create(path) {
            Ok(mut file) => {
                if let Err(r) = file.write_all(value) {
                    log::warn!("Unable to write file content {:?}", r);
                    false
                } else {
                    true
                }
            }
            Err(r) => {
                log::warn!("Unable to save file {:?}", r);
                false
            }
        }
    }

    fn remove_key(&mut self, name: &str) {
        let path = self.get_shared_object_path(name);
        if !Self::is_path_allowed(&path) {
            return;
        }
        let _ = fs::remove_file(path);
    }
}
