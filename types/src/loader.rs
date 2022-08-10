use crate::backend::render::{determine_jpeg_tag_format, JpegTagFormat};
use gc_arena::Collect;
use std::fmt;
use swf::read::read_compression_type;
use thiserror::Error;

/// Enumeration of all content types that `Loader` can handle.
///
/// This is a superset of `JpegTagFormat`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentType {
    Swf,
    Jpeg,
    Png,
    Gif,
    Unknown,
}

impl From<JpegTagFormat> for ContentType {
    #[inline]
    fn from(jtf: JpegTagFormat) -> Self {
        match jtf {
            JpegTagFormat::Jpeg => Self::Jpeg,
            JpegTagFormat::Png => Self::Png,
            JpegTagFormat::Gif => Self::Gif,
            JpegTagFormat::Unknown => Self::Unknown,
        }
    }
}

impl fmt::Display for ContentType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Swf => write!(f, "SWF"),
            Self::Jpeg => write!(f, "JPEG"),
            Self::Png => write!(f, "PNG"),
            Self::Gif => write!(f, "GIF"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContentType {
    #[inline]
    pub fn sniff(data: &[u8]) -> ContentType {
        if read_compression_type(data).is_ok() {
            ContentType::Swf
        } else {
            determine_jpeg_tag_format(data).into()
        }
    }

    /// Assert that content is of a given type, and error otherwise.
    #[inline]
    pub fn expect(self, expected: Self) -> Result<Self, Error> {
        if self == expected {
            Ok(self)
        } else {
            Err(Error::UnexpectedData(expected, self))
        }
    }
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub enum DataFormat {
    Binary,
    Text,
    Variables,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Load cancelled")]
    Cancelled,

    #[error("Non-root-movie loader spawned as root movie loader")]
    NotRootMovieLoader,

    #[error("Non-movie loader spawned as movie loader")]
    NotMovieLoader,

    #[error("Non-form loader spawned as form loader")]
    NotFormLoader,

    #[error("Non-load vars loader spawned as load vars loader")]
    NotLoadVarsLoader,

    #[error("Non-data loader spawned as data loader")]
    NotLoadDataLoader,

    #[error("Could not fetch: {0}")]
    FetchError(String),

    #[error("Invalid SWF")]
    InvalidSwf(#[from] crate::tag_utils::Error),

    #[error("Unexpected content of type {1}, expected {0}")]
    UnexpectedData(ContentType, ContentType),

    // TODO: We can't support lifetimes on this error object yet (or we'll need some backends inside
    // the GC arena). We're losing info here. How do we fix that?
    #[error("Error running avm1 script: {0}")]
    Avm1Error(String),
}

/// The completion status of a `Loader` loading a movie.
#[derive(Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
pub enum LoaderStatus {
    /// The movie hasn't been loaded yet.
    Pending,
    /// The movie loaded successfully.
    Succeeded,
    /// An error occurred while loading the movie.
    Failed,
}
