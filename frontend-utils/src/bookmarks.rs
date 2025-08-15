mod read;
mod write;
pub use read::read_bookmarks;
pub use write::BookmarksWriter;

use url::Url;

#[derive(Debug, PartialEq)]
pub struct Bookmark {
    pub url: Url,
    pub name: String,
}

impl Bookmark {
    pub fn is_invalid(&self) -> bool {
        self.url.as_str() == crate::INVALID_URL
    }
}

pub type Bookmarks = Vec<Bookmark>;
