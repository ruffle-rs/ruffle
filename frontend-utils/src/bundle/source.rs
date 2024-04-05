use std::io::{Error, Read};
use std::path::{Path, PathBuf};

pub mod directory;

trait BundleSourceImpl {
    type Read: Read;

    /// Reads any file from the bundle.
    fn read_file(&self, path: &str) -> Result<Self::Read, Error>;

    /// Reads a file specifically from the content directory of the bundle.
    fn read_content(&self, path: &str) -> Result<Self::Read, Error>;
}

pub enum BundleSource {
    Directory(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum BundleSourceError {
    #[error("Unknown bundle source")]
    UnknownSource,
}

impl BundleSource {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, BundleSourceError> {
        let path = path.as_ref();
        if path.is_dir() {
            return Ok(Self::Directory(path.to_owned()));
        }

        Err(BundleSourceError::UnknownSource)
    }

    /// Reads any file from the bundle.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, Error> {
        match self {
            BundleSource::Directory(directory) => {
                let mut file = directory.read_file(path)?;
                let mut data = vec![];
                file.read_to_end(&mut data)?;
                Ok(data)
            }
        }
    }

    /// Reads a file specifically from the content directory of the bundle.
    pub fn read_content(&self, path: &str) -> Result<Vec<u8>, Error> {
        match self {
            BundleSource::Directory(directory) => {
                let mut file = directory.read_content(path)?;
                let mut data = vec![];
                file.read_to_end(&mut data)?;
                Ok(data)
            }
        }
    }
}
