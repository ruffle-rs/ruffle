pub mod bookmarks;
pub mod bundle;
pub mod parse;
pub mod write;

pub mod backends;
pub mod content;

use std::borrow::Cow;
use url::Url;

pub fn url_to_readable_name(url: &Url) -> Cow<'_, str> {
    let name = url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or_else(|| url.as_str());

    urlencoding::decode(name).unwrap_or(Cow::Borrowed(name))
}
