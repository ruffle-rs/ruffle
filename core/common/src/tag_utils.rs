use crate::sandbox::SandboxType;

use gc_arena::Collect;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use swf::{Fixed8, HeaderExt, Rectangle, Twips};
use url::Url;

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

    /// Construct a movie based on the contents of the SWF datastream.
    pub fn from_data(
        swf_data: &[u8],
        url: String,
        loader_url: Option<String>,
    ) -> Result<Self, swf::error::Error> {
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
    pub movie: Arc<SwfMovie>,
    pub start: usize,
    pub end: usize,
}

impl From<Arc<SwfMovie>> for SwfSlice {
    fn from(movie: Arc<SwfMovie>) -> Self {
        let end = movie.data().len();

        Self {
            movie,
            start: 0,
            end,
        }
    }
}

impl AsRef<[u8]> for SwfSlice {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data()
    }
}

impl SwfSlice {
    /// Creates an empty SwfSlice.
    #[inline]
    pub fn empty(movie: Arc<SwfMovie>) -> Self {
        Self {
            movie,
            start: 0,
            end: 0,
        }
    }

    /// Creates an empty SwfSlice of the same movie.
    #[inline]
    pub fn copy_empty(&self) -> Self {
        Self::empty(self.movie.clone())
    }

    /// Construct a new SwfSlice from a regular slice.
    ///
    /// This function returns None if the given slice is not a subslice of the
    /// current slice.
    pub fn to_subslice(&self, slice: &[u8]) -> Self {
        let self_pval = self.movie.data().as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a movie subslice.
    ///
    /// This function allows subslices outside the current slice to be formed,
    /// as long as they are valid subslices of the movie itself.
    pub fn to_unbounded_subslice(&self, slice: &[u8]) -> Self {
        let self_pval = self.movie.data().as_ptr() as usize;
        let self_len = self.movie.data().len();
        let slice_pval = slice.as_ptr() as usize;

        if self_pval <= slice_pval && slice_pval < (self_pval + self_len) {
            Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a Reader and a size.
    ///
    /// This is intended to allow constructing references to the contents of a
    /// given SWF tag. You just need the current reader and the size of the tag
    /// you want to reference.
    ///
    /// The returned slice may or may not be a subslice of the current slice.
    /// If the resulting slice would be outside the bounds of the underlying
    /// movie, or the given reader refers to a different underlying movie, this
    /// function returns an empty slice.
    pub fn resize_to_reader(&self, reader: &mut SwfStream<'_>, size: usize) -> Self {
        if self.movie.data().as_ptr() as usize <= reader.get_ref().as_ptr() as usize
            && (reader.get_ref().as_ptr() as usize)
                < self.movie.data().as_ptr() as usize + self.movie.data().len()
        {
            let outer_offset =
                reader.get_ref().as_ptr() as usize - self.movie.data().as_ptr() as usize;
            let new_start = outer_offset;
            let new_end = outer_offset + size;

            let len = self.movie.data().len();

            if new_start < len && new_end < len {
                Self {
                    movie: self.movie.clone(),
                    start: new_start,
                    end: new_end,
                }
            } else {
                self.copy_empty()
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a start and an end.
    ///
    /// The start and end values will be relative to the current slice.
    /// Furthermore, this function will yield an empty slice if the calculated slice
    /// would be invalid (e.g. negative length) or would extend past the end of
    /// the current slice.
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Self {
        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_start <= new_end {
            if let Some(result) = self.movie.data().get(new_start..new_end) {
                self.to_subslice(result)
            } else {
                self.copy_empty()
            }
        } else {
            self.copy_empty()
        }
    }

    /// Convert the SwfSlice into a standard data slice.
    pub fn data(&self) -> &[u8] {
        &self.movie.data()[self.start..self.end]
    }

    /// Get the version of the SWF this data comes from.
    pub fn version(&self) -> u8 {
        self.movie.header().version()
    }

    /// Checks if this slice is empty
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// Construct a reader for this slice.
    ///
    /// The `from` parameter is the offset to start reading the slice from.
    pub fn read_from(&self, from: u64) -> swf::read::Reader<'_> {
        swf::read::Reader::new(&self.data()[from as usize..], self.movie.version())
    }

    /// Get the length of the SwfSlice.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
