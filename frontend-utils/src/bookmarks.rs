mod read;
mod write;
pub use read::read_bookmarks;
pub use write::BookmarksWriter;

use crate::content::ContentDescriptor;

#[derive(Debug, PartialEq)]
pub struct Bookmark {
    pub content_descriptor: ContentDescriptor,
    pub name: String,
}

impl Bookmark {
    pub fn is_invalid(&self) -> bool {
        self.content_descriptor.url.as_str() == crate::INVALID_URL
    }
}

pub type Bookmarks = Vec<Bookmark>;
