mod read;
mod write;

pub use read::read_recents;
pub use write::RecentsWriter;

use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct Recent {
    pub url: Url,
    pub name: String,
}

impl Recent {
    pub fn is_invalid(&self) -> bool {
        self.url.as_str() == crate::INVALID_URL
    }

    /// Checks if a recent entry is available.
    ///
    /// If the URL is local file, it will be checked if it exists, otherwise returns `true`.
    #[cfg(feature = "fs")]
    pub fn is_available(&self) -> bool {
        if self.url.scheme() == "file" {
            return match self.url.to_file_path() {
                Ok(path) => path.exists(),
                Err(()) => false,
            };
        }

        true
    }
}

/// Recent entries, stored from oldest to newest.
pub type Recents = Vec<Recent>;
