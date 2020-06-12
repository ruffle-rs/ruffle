use std::string::ToString;
use ruffle_core::backend::storage::StorageBackend;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::{Write, Read};

pub struct DiskStorageBackend {
    base_path: String
}

impl DiskStorageBackend {
    pub fn new() -> Self {
        let bp = "/home/cub3d/.local/share/ruffle/".to_string();

        // Create a base dir if one doesn't exist yet
        let base_path = Path::new(&bp);
        if !base_path.exists() {
            log::info!("Creating storage dir");
            if let Err(r) = fs::create_dir_all(base_path) {
                log::warn!("Unable to create storage dir {}", r);
            }
        }

        DiskStorageBackend {
            base_path: bp
        }
    }
}


impl StorageBackend for DiskStorageBackend {
    fn get_string(&self, name: String) -> Option<String> {
        let base_path = Path::new(&self.base_path);
        let full_path = base_path.join(Path::new(&name));

        match File::open(full_path.clone()) {
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

    fn put_string(&mut self, name: String, value: String) {
        let base_path = Path::new(&self.base_path);
        let full_path = base_path.join(Path::new(&name));

        match File::create(full_path.clone()) {
            Ok(mut file) => {
                if let Err(r) = file.write_all(value.as_bytes()) {
                    log::warn!("Unable to write file content {:?}", r)
                }
            }
            Err(r) =>  log::warn!("Unable to save file {:?}", r)
        }

        log::info!("[storage] Saved {} to {:?}", value, full_path);
    }
}

