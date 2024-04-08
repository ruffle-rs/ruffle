use crate::bundle::info::BUNDLE_INFORMATION_FILENAME;
use crate::bundle::source::zip::ZipSource;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

pub mod directory;
mod zip;

trait BundleSourceImpl {
    type Read: Read;

    /// Reads any file from the bundle.
    fn read_file(&self, path: &str) -> Result<Self::Read, Error>;

    /// Reads a file specifically from the content directory of the bundle.
    fn read_content(&self, path: &str) -> Result<Self::Read, Error>;
}

pub enum BundleSource {
    Directory(PathBuf),
    ZipFile(ZipSource<File>),
}

#[derive(Debug, thiserror::Error)]
pub enum BundleSourceError {
    #[error("Unknown bundle source")]
    UnknownSource,

    #[error("Invalid or corrupt zip")]
    InvalidZip,

    #[error("IO error opening file")]
    Io(#[from] Error),
}

impl BundleSource {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, BundleSourceError> {
        let path = path.as_ref();

        // Opening a directory which happens to contain a ruffle-bundle.toml file, the bundle is this directory
        if path.is_dir() && path.join(BUNDLE_INFORMATION_FILENAME).is_file() {
            return Ok(Self::Directory(path.to_owned()));
        }

        if path.is_file() {
            // Opening a ruffle-bundle.toml, the bundle is the parent directory
            if path.file_name() == Some(OsStr::new(BUNDLE_INFORMATION_FILENAME)) {
                if let Some(parent) = path.parent() {
                    return Ok(Self::Directory(parent.to_owned()));
                }
            }

            // Opening a .ruf file, the bundle is that file viewed as a zip
            if path.extension() == Some(OsStr::new("ruf")) {
                return if let Ok(zip) = ZipSource::open(File::open(path)?) {
                    Ok(Self::ZipFile(zip))
                } else {
                    Err(BundleSourceError::InvalidZip)
                };
            }
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
            BundleSource::ZipFile(zip) => zip.read_file(path).map(|cursor| cursor.into_inner()),
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
            BundleSource::ZipFile(zip) => zip.read_content(path).map(|cursor| cursor.into_inner()),
        }
    }
}
