mod read;
mod write;

pub use read::read_recents;
pub use write::RecentsWriter;

use crate::content::ContentDescriptor;

#[derive(Clone, Debug, PartialEq)]
pub struct Recent {
    pub content_descriptor: ContentDescriptor,
    pub name: String,
}

impl Recent {
    pub fn is_invalid(&self) -> bool {
        self.content_descriptor.url.as_str() == crate::INVALID_URL
    }

    /// Checks if a recent entry is available.
    ///
    /// If the URL is local file, it will be checked if it exists, otherwise returns `true`.
    #[cfg(feature = "fs")]
    pub fn is_available(&self) -> bool {
        if self.content_descriptor.url.scheme() == "file" {
            return match self.content_descriptor.url.to_file_path() {
                Ok(path) => path.exists(),
                Err(()) => false,
            };
        }

        true
    }
}

/// Recent entries, stored from oldest to newest.
pub type Recents = Vec<Recent>;
