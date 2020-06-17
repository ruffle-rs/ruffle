use ruffle_core::backend::storage::StorageBackend;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct DiskStorageBackend {
    base_path: PathBuf,
}

impl DiskStorageBackend {
    pub fn new(scope: &Path) -> Self {
        let base_path = dirs::data_local_dir()
            .unwrap()
            .join(Path::new("ruffle"))
            .join(scope);

        // Create a base dir if one doesn't exist yet
        if !&base_path.exists() {
            log::info!("Creating storage dir");
            if let Err(r) = fs::create_dir_all(&base_path) {
                log::warn!("Unable to create storage dir {}", r);
            }
        }

        DiskStorageBackend { base_path }
    }
}

impl StorageBackend for DiskStorageBackend {
    fn get_string(&self, name: &str) -> Option<String> {
        let full_path = self.base_path.join(Path::new(name));

        match File::open(full_path) {
            Ok(mut file) => {
                let mut buffer = String::new();
                if let Err(r) = file.read_to_string(&mut buffer) {
                    log::warn!("Unable to read file content {:?}", r);
                    None
                } else {
                    Some(buffer)
                }
            }
            Err(r) => {
                log::warn!("Unable to open file {:?}", r);
                None
            }
        }
    }

    fn put_string(&mut self, name: &str, value: String) -> bool {
        let full_path = self.base_path.join(Path::new(name));

        match File::create(full_path) {
            Ok(mut file) => {
                if let Err(r) = file.write_all(value.as_bytes()) {
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
        let full_path = self.base_path.join(Path::new(name));
        let _ = fs::remove_file(full_path);
    }
}
