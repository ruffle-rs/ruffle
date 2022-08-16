use gc_arena::Collect;
use std::sync::Arc;
use swf::{Fixed8, HeaderExt, Rectangle, TagCode, Twips};

pub type Error = Box<dyn std::error::Error>;
pub type DecodeResult = Result<(), Error>;
pub type SwfStream<'a> = swf::read::Reader<'a>;

/// An open, fully parsed SWF movie ready to play back, either in a Player or a
/// MovieClip.
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
pub struct SwfMovie {
    /// The SWF header parsed from the data stream.
    header: HeaderExt,

    /// Uncompressed SWF data.
    data: Vec<u8>,

    /// The URL the SWF was downloaded from.
    url: Option<String>,

    /// The URL that triggered the SWF load.
    loader_url: Option<String>,

    /// Any parameters provided when loading this movie (also known as 'flashvars'),
    /// as a list of key-value pairs.
    parameters: Vec<(String, String)>,

    /// The suggest encoding for this SWF.
    encoding: &'static swf::Encoding,

    /// The compressed length of the entire datastream
    compressed_len: usize,
}

impl SwfMovie {
    /// Construct an empty movie.
    pub fn empty(swf_version: u8) -> Self {
        Self {
            header: HeaderExt::default_with_swf_version(swf_version),
            data: vec![],
            url: None,
            loader_url: None,
            parameters: Vec::new(),
            encoding: swf::UTF_8,
            compressed_len: 0,
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
        let url = url::Url::from_file_path(abs_path).map_err(|()| "Invalid SWF URL")?;

        Self::from_data(&data, Some(url.into()), loader_url)
    }

    /// Construct a movie based on the contents of the SWF datastream.
    pub fn from_data(
        swf_data: &[u8],
        url: Option<String>,
        loader_url: Option<String>,
    ) -> Result<Self, Error> {
        let compressed_len = swf_data.len();
        let swf_buf = swf::read::decompress_swf(swf_data)?;
        let encoding = swf::SwfStr::encoding_for_version(swf_buf.header.version());
        Ok(Self {
            header: swf_buf.header,
            data: swf_buf.data,
            url,
            loader_url,
            parameters: Vec::new(),
            encoding,
            compressed_len,
        })
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
        self.header.stage_size().x_max - self.header.stage_size().x_min
    }

    /// The height of the movie in twips.
    pub fn height(&self) -> Twips {
        self.header.stage_size().y_max - self.header.stage_size().y_min
    }

    /// Get the URL this SWF was fetched from.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
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

    pub fn uncompressed_len(&self) -> u32 {
        self.header.uncompressed_len()
    }

    pub fn is_action_script_3(&self) -> bool {
        self.header.is_action_script_3()
    }

    pub fn stage_size(&self) -> &Rectangle {
        self.header.stage_size()
    }

    pub fn num_frames(&self) -> u16 {
        self.header.num_frames()
    }

    pub fn frame_rate(&self) -> Fixed8 {
        self.header.frame_rate()
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

    /// Construct a new SwfSlice from a regular slice.
    ///
    /// This function returns None if the given slice is not a subslice of the
    /// current slice.
    pub fn to_subslice(&self, slice: &[u8]) -> Option<Self> {
        let self_pval = self.movie.data().as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Some(Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            })
        } else {
            None
        }
    }

    /// Construct a new SwfSlice from a movie subslice.
    ///
    /// This function allows subslices outside the current slice to be formed,
    /// as long as they are valid subslices of the movie itself.
    pub fn to_unbounded_subslice(&self, slice: &[u8]) -> Option<Self> {
        let self_pval = self.movie.data().as_ptr() as usize;
        let self_len = self.movie.data().len();
        let slice_pval = slice.as_ptr() as usize;

        if self_pval <= slice_pval && slice_pval < (self_pval + self_len) {
            Some(Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            })
        } else {
            None
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
    /// function returns None.
    pub fn resize_to_reader(&self, reader: &mut SwfStream<'_>, size: usize) -> Option<Self> {
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
                Some(Self {
                    movie: self.movie.clone(),
                    start: new_start,
                    end: new_end,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Construct a new SwfSlice from a start and an end.
    ///
    /// The start and end values will be relative to the current slice.
    /// Furthermore, this function will yield None if the calculated slice
    /// would be invalid (e.g. negative length) or would extend past the end of
    /// the current slice.
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Option<Self> {
        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_start <= new_end {
            self.to_subslice(self.movie.data().get(new_start..new_end)?)
        } else {
            None
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

    /// Construct a reader for this slice.
    ///
    /// The `from` parameter is the offset to start reading the slice from.
    pub fn read_from(&self, from: u64) -> swf::read::Reader<'_> {
        swf::read::Reader::new(&self.data()[from as usize..], self.movie.version())
    }
}

pub fn decode_tags<'a, F>(
    reader: &mut SwfStream<'a>,
    mut tag_callback: F,
    stop_tag: TagCode,
) -> Result<(), Error>
where
    F: for<'b> FnMut(&'b mut SwfStream<'a>, TagCode, usize) -> DecodeResult,
{
    loop {
        let (tag_code, tag_len) = reader.read_tag_code_and_length()?;
        if tag_len > reader.get_ref().len() {
            log::error!("Unexpected EOF when reading tag");
            *reader.get_mut() = &reader.get_ref()[reader.get_ref().len()..];
            break;
        }

        let tag_slice = &reader.get_ref()[..tag_len];
        let end_slice = &reader.get_ref()[tag_len..];
        if let Some(tag) = TagCode::from_u16(tag_code) {
            *reader.get_mut() = tag_slice;
            let result = tag_callback(reader, tag, tag_len);

            if let Err(e) = result {
                log::error!("Error running definition tag: {:?}, got {}", tag, e);
            }

            if stop_tag == tag {
                *reader.get_mut() = end_slice;
                break;
            }
        } else {
            log::warn!("Unknown tag code: {:?}", tag_code);
        }

        *reader.get_mut() = end_slice;
    }

    Ok(())
}
