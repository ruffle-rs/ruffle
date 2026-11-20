use ruffle_core::backend::storage::StorageBackend;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Component, PathBuf};

pub struct DiskStorageBackend {
    shared_objects_path: PathBuf,
}

impl DiskStorageBackend {
    pub fn new(shared_objects_path: PathBuf) -> Self {
        // Create a base dir if one doesn't exist yet
        if !shared_objects_path.exists() {
            tracing::info!("Creating storage dir");
            if let Err(r) = fs::create_dir_all(&shared_objects_path) {
                tracing::warn!("Unable to create storage dir {}", r);
            }
        }

        DiskStorageBackend {
            shared_objects_path,
        }
    }

    /// Returns the path of the shared object file, or `None` when it would not
    /// refer to a file inside the shared objects directory.
    ///
    /// Only normal components are allowed in the name, which rejects `..`,
    /// absolute paths, and (on Windows) drive and UNC prefixes.
    fn get_shared_object_path(&self, name: &str) -> Option<PathBuf> {
        let relative_path = PathBuf::from(format!("{name}.sol"));
        if !relative_path
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        {
            tracing::warn!("Invalid shared object name \"{}\"", name);
            return None;
        }
        Some(self.shared_objects_path.join(relative_path))
    }
}

impl StorageBackend for DiskStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.get_shared_object_path(name)?;
        match fs::read(path) {
            Ok(data) => Some(data),
            Err(e) => {
                tracing::warn!("Unable to read file \"{}\": {:?}", name, e);
                None
            }
        }
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        let Some(path) = self.get_shared_object_path(name) else {
            return false;
        };
        if let Some(parent_dir) = path.parent()
            && !parent_dir.exists()
            && let Err(r) = fs::create_dir_all(parent_dir)
        {
            tracing::warn!("Unable to create storage dir {}", r);
            return false;
        }

        match File::create(path) {
            Ok(mut file) => {
                if let Err(r) = file.write_all(value) {
                    tracing::warn!("Unable to write file content {:?}", r);
                    false
                } else {
                    true
                }
            }
            Err(r) => {
                tracing::warn!("Unable to save file {:?}", r);
                false
            }
        }
    }

    fn remove_key(&mut self, name: &str) {
        let Some(path) = self.get_shared_object_path(name) else {
            return;
        };
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> DiskStorageBackend {
        DiskStorageBackend {
            shared_objects_path: PathBuf::from("base"),
        }
    }

    #[test]
    fn shared_object_path_inside_directory() {
        assert_eq!(
            backend().get_shared_object_path("localhost/game.swf/so"),
            Some(PathBuf::from("base/localhost/game.swf/so.sol"))
        );
    }

    #[test]
    fn shared_object_path_outside_directory() {
        let backend = backend();
        assert_eq!(backend.get_shared_object_path("../so"), None);
        assert_eq!(backend.get_shared_object_path("localhost/../../so"), None);
        assert_eq!(backend.get_shared_object_path("/so"), None);
        assert_eq!(backend.get_shared_object_path("./so"), None);
        #[cfg(windows)]
        {
            assert_eq!(backend.get_shared_object_path("C:/so"), None);
            assert_eq!(backend.get_shared_object_path("C:so"), None);
            assert_eq!(
                backend.get_shared_object_path("\\\\server\\share\\so"),
                None
            );
        }
    }
}
