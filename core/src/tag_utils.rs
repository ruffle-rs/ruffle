use gc_arena::Collect;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::Weak;
use swf::{CharacterId, Fixed8, HeaderExt, Rectangle, TagCode, Twips};
use thiserror::Error;
use url::Url;

use crate::sandbox::SandboxType;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't read SWF: {0}")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("Couldn't register bitmap: {0}")]
    InvalidBitmap(#[from] ruffle_render::error::Error),

    #[error("Couldn't register font: {0}")]
    InvalidFont(#[from] ttf_parser::FaceParsingError),

    #[error("Attempted to set symbol classes on movie without any")]
    NoSymbolClasses,

    #[error("Attempted to preload video frames into non-video character {0}")]
    PreloadVideoIntoInvalidCharacter(CharacterId),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid SWF url")]
    InvalidSwfUrl,
}

/// Whether or not to end tag decoding.
pub enum ControlFlow {
    /// Stop decoding after this tag.
    Exit,

    /// Continue decoding the next tag.
    Continue,
}

pub type DecodeResult = Result<ControlFlow, Error>;
pub type SwfStream<'a> = swf::read::Reader<'a>;

/// An open, fully parsed SWF movie ready to play back, either in a Player or a
/// MovieClip.
#[derive(Clone, Collect)]
#[collect(require_static)]
pub struct SwfMovie {
    /// The SWF header parsed from the data stream.
    header: HeaderExt,

    /// Uncompressed SWF data.
    data: Vec<u8>,

    /// The URL the SWF was downloaded from.
    url: String,

    /// The URL that triggered the SWF load.
    loader_url: Option<String>,

    /// Any parameters provided when loading this movie (also known as 'flashvars'),
    /// as a list of key-value pairs.
    parameters: Vec<(String, String)>,

    /// The suggest encoding for this SWF.
    encoding: &'static swf::Encoding,

    /// The compressed length of the entire datastream
    compressed_len: usize,

    /// Whether this SwfMovie actually represents a loaded movie or fills in for
    /// something else, like an loaded image, filler movie, or error state.
    is_movie: bool,

    /// Security sandbox type enforced for this movie.
    ///
    /// It absolutely cannot be changed after constructing
    /// the object in order to ensure proper sandboxing.
    sandbox_type: SandboxType,
}

impl SwfMovie {
    /// Construct an empty movie.
    pub fn empty(swf_version: u8, loader_url: Option<String>) -> Self {
        let url = "file:///".to_string();
        let header = HeaderExt::default_with_swf_version(swf_version);

        // TODO What sandbox type should we use here?
        let sandbox_type = SandboxType::infer(url.as_str(), &header);
        Self {
            header,
            data: vec![],
            url,
            loader_url,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            compressed_len: 0,
            is_movie: false,
            sandbox_type,
        }
    }

    /// Construct an empty movie with a fake `compressed_len`.
    /// This is used by `Loader` when firing an initial `progress` event:
    /// `LoaderInfo.bytesTotal` is set to the actual value, but no data is available,
    /// and `LoaderInfo.parameters` is empty.
    pub fn fake_with_compressed_len(
        swf_version: u8,
        loader_url: Option<String>,
        compressed_len: usize,
    ) -> Self {
        let url = "file:///".to_string();
        let header = HeaderExt::default_with_swf_version(swf_version);

        // TODO What sandbox type should we use here?
        let sandbox_type = SandboxType::infer(url.as_str(), &header);
        Self {
            header,
            compressed_len,
            data: Vec::new(),
            url,
            loader_url,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            is_movie: false,
            sandbox_type,
        }
    }

    /// Like `fake_with_compressed_len`, but uses actual data.
    /// This is used when loading a Bitmap to expose the underlying content
    pub fn fake_with_compressed_data(
        swf_version: u8,
        loader_url: Option<String>,
        compressed_data: Vec<u8>,
    ) -> Self {
        let url = "file:///".to_string();
        let header = HeaderExt::default_with_swf_version(swf_version);

        // TODO What sandbox type should we use here?
        let sandbox_type = SandboxType::infer(url.as_str(), &header);
        Self {
            header,
            compressed_len: compressed_data.len(),
            data: compressed_data,
            url,
            loader_url,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            is_movie: false,
            sandbox_type,
        }
    }

    /// Constructs the error state movie stub in which some attributes have certain
    /// error values to signal that no valid file could be loaded.
    ///
    /// This happens if no file could be loaded or if the loaded content is no valid
    /// supported content.
    pub fn error_movie(movie_url: String) -> Self {
        let header = HeaderExt::default_error_header();

        // TODO What sandbox type should we use here?
        let sandbox_type = SandboxType::infer(movie_url.as_str(), &header);
        Self {
            header,
            data: vec![],
            url: movie_url,
            loader_url: None,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            compressed_len: 0,
            is_movie: false,
            sandbox_type,
        }
    }

    /// Utility method to construct a movie from a file on disk.
    #[cfg(any(unix, windows, target_os = "redox"))]
    pub fn from_path<P: AsRef<std::path::Path>>(
        path: P,
        loader_url: Option<String>,
    ) -> Result<Self, Error> {
        let data = std::fs::read(&path)?;

        let abs_path = path.as_ref().canonicalize()?;
        let url = url::Url::from_file_path(abs_path).map_err(|()| Error::InvalidSwfUrl)?;

        Self::from_data(&data, url.into(), loader_url)
    }

    /// Construct a movie based on the contents of the SWF datastream.
    pub fn from_data(
        swf_data: &[u8],
        url: String,
        loader_url: Option<String>,
    ) -> Result<Self, Error> {
        let compressed_len = swf_data.len();
        let swf_buf = swf::read::decompress_swf(swf_data)?;
        let encoding = swf::SwfStr::encoding_for_version(swf_buf.header.version());
        let sandbox_type = SandboxType::infer(url.as_str(), &swf_buf.header);
        let mut movie = Self {
            header: swf_buf.header,
            data: swf_buf.data,
            url,
            loader_url,
            parameters: Vec::new(),
            encoding,
            compressed_len,
            is_movie: true,
            sandbox_type,
        };
        movie.append_parameters_from_url();
        Ok(movie)
    }

    /// Construct a movie based on a loaded image (JPEG, GIF or PNG).
    pub fn from_loaded_image(url: String, length: usize) -> Self {
        let header = HeaderExt::default_with_uncompressed_len(length as i32);
        let sandbox_type = SandboxType::infer(url.as_str(), &header);
        let mut movie = Self {
            header,
            data: vec![],
            url,
            loader_url: None,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            compressed_len: length,
            is_movie: false,
            sandbox_type,
        };
        movie.append_parameters_from_url();
        movie
    }

    fn append_parameters_from_url(&mut self) {
        match Url::parse(&self.url) {
            Ok(url) => {
                for (key, value) in url.query_pairs() {
                    self.parameters.push((key.into_owned(), value.into_owned()));
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to parse loader URL when extracting query parameters: {}",
                    e
                );
            }
        }
    }

    pub fn header(&self) -> &HeaderExt {
        &self.header
    }

    /// Get the version of the SWF.
    pub fn version(&self) -> u8 {
        self.header.version()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the suggested string encoding for the given SWF version.
    /// For SWF version 6 and higher, this is always UTF-8.
    /// For SWF version 5 and lower, this is locale-dependent,
    /// and we default to WINDOWS-1252.
    pub fn encoding(&self) -> &'static swf::Encoding {
        self.encoding
    }

    /// The width of the movie in twips.
    pub fn width(&self) -> Twips {
        self.header.stage_size().width()
    }

    /// The height of the movie in twips.
    pub fn height(&self) -> Twips {
        self.header.stage_size().height()
    }

    /// Get the URL this SWF was fetched from.
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    /// Get the URL that triggered the fetch of this SWF.
    pub fn loader_url(&self) -> Option<&str> {
        self.loader_url.as_deref()
    }

    pub fn parameters(&self) -> &[(String, String)] {
        &self.parameters
    }

    pub fn append_parameters(&mut self, params: impl IntoIterator<Item = (String, String)>) {
        self.parameters.extend(params);
    }

    pub fn compressed_len(&self) -> usize {
        self.compressed_len
    }

    pub fn uncompressed_len(&self) -> i32 {
        self.header.uncompressed_len()
    }

    pub fn is_action_script_3(&self) -> bool {
        self.header.is_action_script_3()
    }

    pub fn stage_size(&self) -> &Rectangle<Twips> {
        self.header.stage_size()
    }

    pub fn num_frames(&self) -> u16 {
        self.header.num_frames()
    }

    pub fn frame_rate(&self) -> Fixed8 {
        self.header.frame_rate()
    }

    pub fn is_movie(&self) -> bool {
        self.is_movie
    }

    pub fn sandbox_type(&self) -> SandboxType {
        self.sandbox_type
    }
}

impl Debug for SwfMovie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwfMovie")
            .field("header", &self.header)
            .field("data", &self.data.len())
            .field("url", &self.url)
            .field("loader_url", &self.loader_url)
            .field("parameters", &self.parameters)
            .field("encoding", &self.encoding)
            .field("compressed_len", &self.compressed_len)
            .field("is_movie", &self.is_movie)
            .field("sandbox_type", &self.sandbox_type)
            .finish()
    }
}

/// A shared-ownership reference to some portion of an SWF datastream.
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct SwfSlice {
    /// An optional strong reference to the movie.
    /// This is `Some` only when this slice is responsible for keeping the movie data alive.
    /// Sub-slices created from this one will have `None` and rely on this original reference.
    #[collect(require_static)]
    movie_owner: Option<Arc<SwfMovie>>,

    #[collect(require_static)]
    pub movie: Weak<SwfMovie>,
    pub start: usize,
    pub end: usize,
}

impl From<Arc<SwfMovie>> for SwfSlice {
    /// Creates a new `SwfSlice` that encompasses the entire movie data and takes ownership.
    fn from(movie: Arc<SwfMovie>) -> Self {
        let end = movie.data().len();
        Self {
            movie: Arc::downgrade(&movie),
            movie_owner: Some(movie),
            start: 0,
            end,
        }
    }
}

impl SwfSlice {
    /// Helper to get a strong reference to the movie, either from the owner or by upgrading the weak ref.
    pub fn get_movie_arc(&self) -> Option<Arc<SwfMovie>> {
        if let Some(owner) = &self.movie_owner {
            return Some(owner.clone());
        }
        self.movie.upgrade()
    }

    /// Creates an empty `SwfSlice` from a given movie `Arc`, taking ownership to keep it alive.
    #[inline]
    pub fn empty(movie: Arc<SwfMovie>) -> Self {
        Self {
            movie: Arc::downgrade(&movie),
            movie_owner: Some(movie),
            start: 0,
            end: 0,
        }
    }

    /// Creates an empty `SwfSlice` that refers to the same movie as `self`, but does not own the data.
    #[inline]
    pub fn copy_empty(&self) -> Self {
        Self {
            movie_owner: None,
            movie: self.movie.clone(),
            start: 0,
            end: 0,
        }
    }

    /// Executes a closure with the slice's data, if the parent movie still exists.
    ///
    /// This is the primary safe way to access the byte data of the slice.
    /// It returns `None` if the `SwfMovie` has been dropped.
    pub fn with_data<T>(&self, f: impl FnOnce(&[u8]) -> T) -> Option<T> {
        self.get_movie_arc()
            .map(|movie| f(&movie.data()[self.start..self.end]))
    }

    /// Get the version of the SWF this data comes from.
    ///
    /// Returns `None` if the parent movie has been deallocated.
    pub fn version(&self) -> Option<u8> {
        self.get_movie_arc().map(|movie| movie.version())
    }

    /// Construct a new `SwfSlice` from a regular slice.
    ///
    /// This function returns an empty slice if the given `slice` is not a valid
    /// subslice of this `SwfSlice`'s movie data, or if the movie has been deallocated.
    pub fn to_subslice(&self, slice: &[u8]) -> Self {
        if let Some(movie) = self.get_movie_arc() {
            let movie_ptr = movie.data().as_ptr() as usize;
            let slice_ptr = slice.as_ptr() as usize;

            // Check if the slice lives within the movie's data range.
            if slice_ptr >= movie_ptr
                && (slice_ptr + slice.len()) <= (movie_ptr + movie.data().len())
            {
                let new_start = slice_ptr - movie_ptr;
                let new_end = new_start + slice.len();
                // Check if the new slice is a sub-slice of the *current* slice.
                if new_start >= self.start && new_end <= self.end {
                    return Self {
                        movie_owner: None,
                        movie: Arc::downgrade(&movie),
                        start: new_start,
                        end: new_end,
                    };
                }
            }
        }
        self.copy_empty()
    }

    /// Construct a new `SwfSlice` from a movie subslice.
    ///
    /// This is similar to `to_subslice`, but allows creating a slice that is
    /// outside the bounds of the current slice, as long as it's within the
    /// bounds of the parent `SwfMovie`'s data.
    pub fn to_unbounded_subslice(&self, slice: &[u8]) -> Self {
        if let Some(movie) = self.get_movie_arc() {
            let movie_ptr = movie.data().as_ptr() as usize;
            let slice_ptr = slice.as_ptr() as usize;

            if slice_ptr >= movie_ptr
                && (slice_ptr + slice.len()) <= (movie_ptr + movie.data().len())
            {
                return Self {
                    movie_owner: None,
                    movie: Arc::downgrade(&movie),
                    start: slice_ptr - movie_ptr,
                    end: (slice_ptr - movie_ptr) + slice.len(),
                };
            }
        }
        self.copy_empty()
    }

    /// Construct a new `SwfSlice` from a `Reader` and a size.
    ///
    /// This is intended for creating a slice that refers to the content of an SWF tag.
    /// Returns an empty slice if the bounds are invalid or the movie is deallocated.
    pub fn resize_to_reader(&self, reader: &SwfStream<'_>, size: usize) -> Self {
        if let Some(movie) = self.get_movie_arc() {
            let movie_data = movie.data();
            let movie_ptr = movie_data.as_ptr() as usize;
            let reader_ptr = reader.get_ref().as_ptr() as usize;

            if reader_ptr >= movie_ptr && (reader_ptr + size) <= (movie_ptr + movie_data.len()) {
                let new_start = reader_ptr - movie_ptr;
                let new_end = new_start + size;
                return Self {
                    movie_owner: None,
                    movie: Arc::downgrade(&movie),
                    start: new_start,
                    end: new_end,
                };
            }
        }
        self.copy_empty()
    }

    /// Construct a new `SwfSlice` from a start and an end relative to the current slice.
    ///
    /// Yields an empty slice if the new bounds are invalid or would extend past
    /// the end of the current slice.
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Self {
        if start > end {
            return self.copy_empty();
        }

        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_end > self.end {
            return self.copy_empty();
        }

        Self {
            movie_owner: None,
            movie: self.movie.clone(),
            start: new_start,
            end: new_end,
        }
    }

    /// Checks if this slice has a length of zero.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the length of the `SwfSlice`.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Executes a closure with a `swf::Reader` for this slice's data.
    ///
    /// The `from` parameter is the offset to start reading the slice from.
    /// Returns `None` if the `SwfMovie` has been dropped.
    pub fn with_reader_from<T>(
        &self,
        from: u64,
        f: impl FnOnce(&mut swf::read::Reader<'_>) -> T,
    ) -> Option<T> {
        self.with_data(|data| {
            let reader_data = data.get(from as usize..).unwrap_or_default();
            // We can safely unwrap `version` here because we are inside `with_data`
            let version = self.version().unwrap();
            let mut reader = swf::read::Reader::new(reader_data, version);
            f(&mut reader)
        })
    }
}
/// Decode tags from a SWF stream reader.
///
/// The given `tag_callback` will be called for each decoded tag. It will be
/// provided with the stream to read from, the tag code read, and the tag's
/// size. The callback is responsible for (optionally) parsing the contents of
/// the tag; otherwise, it will be skipped.
///
/// Decoding will terminate when the following conditions occur:
///
///  * The `tag_callback` calls for the decoding to finish.
///  * The decoder encounters a tag longer than the underlying SWF slice
///    (indicated by returning false)
///  * The SWF stream is otherwise corrupt or unreadable (indicated as an error
///    result)
///
/// Decoding will also log tags longer than the SWF slice, error messages
/// yielded from the tag callback, and unknown tags. It will *only* return an
/// error message if the SWF tag itself could not be parsed. Other forms of
/// irregular decoding will be signalled by returning false.
pub fn decode_tags<'a, F>(reader: &mut SwfStream<'a>, mut tag_callback: F) -> Result<bool, Error>
where
    F: for<'b> FnMut(&'b mut SwfStream<'a>, TagCode, usize) -> Result<ControlFlow, Error>,
{
    loop {
        // This check handles an empty reader, which can happen if the loop reaches the end.
        if reader.get_ref().is_empty() {
            break;
        }

        let (tag_code, tag_len) = reader.read_tag_code_and_length()?;
        if tag_len > reader.get_ref().len() {
            tracing::error!("Unexpected EOF when reading tag");
            *reader.get_mut() = &reader.get_ref()[reader.get_ref().len()..];
            return Ok(false);
        }

        let tag_slice = &reader.get_ref()[..tag_len];
        let end_slice = &reader.get_ref()[tag_len..];
        if let Some(tag) = TagCode::from_u16(tag_code) {
            *reader.get_mut() = tag_slice;
            let result = tag_callback(reader, tag, tag_len);

            match result {
                Err(e) => {
                    tracing::error!("Error running definition tag: {:?}, got {}", tag, e)
                }
                Ok(ControlFlow::Exit) => {
                    *reader.get_mut() = end_slice;
                    break;
                }
                Ok(ControlFlow::Continue) => {}
            }
        } else {
            tracing::warn!("Unknown tag code: {:?}", tag_code);
        }

        *reader.get_mut() = end_slice;
    }

    Ok(true)
}
