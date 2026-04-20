use crate::player::LaunchOptions;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_frontend_utils::backends::storage::DiskStorageBackend;
use std::str::FromStr;

#[derive(clap::ValueEnum, Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum StorageBackend {
    #[default]
    Disk,
    Memory,
}

impl FromStr for StorageBackend {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disk" => Ok(StorageBackend::Disk),
            "memory" => Ok(StorageBackend::Memory),
            _ => Err(()),
        }
    }
}

impl StorageBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            StorageBackend::Disk => "disk",
            StorageBackend::Memory => "memory",
        }
    }

    pub fn create_backend(
        self,
        opt: &LaunchOptions,
    ) -> Box<dyn ruffle_core::backend::storage::StorageBackend> {
        match self {
            StorageBackend::Disk => Box::new(DiskStorageBackend::new(opt.save_directory.clone())),
            StorageBackend::Memory => Box::new(MemoryStorageBackend::new()),
        }
    }
}
