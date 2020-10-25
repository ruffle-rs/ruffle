use crate::backend::navigator::url_from_relative_path;
use crate::property_map::PropertyMap;
use gc_arena::Collect;
use std::path::Path;
use std::sync::Arc;
use swf::{Header, TagCode};

pub type Error = Box<dyn std::error::Error>;
pub type DecodeResult = Result<(), Error>;
pub type SwfStream<R> = swf::read::Reader<std::io::Cursor<R>>;

/// An open, fully parsed SWF movie ready to play back, either in a Player or a
/// MovieClip.
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
pub struct SwfMovie {
    /// The SWF header parsed from the data stream.
    header: Header,

    /// Uncompressed SWF data.
    data: Vec<u8>,

    /// The URL the SWF was downloaded from.
    url: Option<String>,

    /// Any parameters provided when loading this movie (also known as 'flashvars')
    parameters: PropertyMap<String>,
}

impl SwfMovie {
    /// Construct an empty movie.
    pub fn empty(swf_version: u8) -> Self {
        Self {
            header: Header {
                version: swf_version,
                compression: swf::Compression::None,
                stage_size: swf::Rectangle::default(),
                frame_rate: 1.0,
                num_frames: 0,
            },
            data: vec![],
            url: None,
            parameters: PropertyMap::new(),
        }
    }

    /// Construct a movie from an existing movie with any particular data on
    /// it.
    ///
    /// Use of this method is discouraged. SWF data should be borrowed or
    /// sliced as necessary to refer to partial sections of a file.
    pub fn from_movie_and_subdata(&self, data: Vec<u8>, source: &SwfMovie) -> Self {
        Self {
            header: self.header.clone(),
            data,
            url: source.url.clone(),
            parameters: source.parameters.clone(),
        }
    }

    /// Utility method to construct a movie from a file on disk.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut url = path.as_ref().to_string_lossy().to_owned().to_string();
        let cwd = std::env::current_dir()?;
        if let Ok(abs_url) = url_from_relative_path(cwd, &url) {
            url = abs_url.into_string();
        }

        let data = std::fs::read(path)?;
        Self::from_data(&data, Some(url))
    }

    /// Construct a movie based on the contents of the SWF datastream.
    pub fn from_data(swf_data: &[u8], url: Option<String>) -> Result<Self, Error> {
        let swf_stream = swf::read::read_swf_header(&swf_data[..])?;
        let header = swf_stream.header;
        let mut reader = swf_stream.reader;

        // Decompress the entire SWF in memory.
        // Sometimes SWFs will have an incorrectly compressed stream,
        // but will otherwise decompress fine up to the End tag.
        // So just warn on this case and try to continue gracefully.
        let data = if header.compression == swf::Compression::Lzma {
            // TODO: The LZMA decoder is still funky.
            // It always errors, and doesn't return all the data if you use read_to_end,
            // but read_exact at least returns the data... why?
            // Does the decoder need to be flushed somehow?
            let mut data = vec![0u8; swf_stream.uncompressed_length];
            let _ = reader.get_mut().read_exact(&mut data);
            data
        } else {
            let mut data = Vec::with_capacity(swf_stream.uncompressed_length);
            if let Err(e) = reader.get_mut().read_to_end(&mut data) {
                return Err(format!("Error decompressing SWF, may be corrupt: {}", e).into());
            }
            data
        };

        Ok(Self {
            header,
            data,
            url,
            parameters: PropertyMap::new(),
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get the version of the SWF.
    pub fn version(&self) -> u8 {
        self.header.version
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn width(&self) -> u32 {
        (self.header.stage_size.x_max - self.header.stage_size.x_min).to_pixels() as u32
    }

    pub fn height(&self) -> u32 {
        (self.header.stage_size.y_max - self.header.stage_size.y_min).to_pixels() as u32
    }

    /// Get the URL this SWF was fetched from.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn parameters(&self) -> &PropertyMap<String> {
        &self.parameters
    }

    pub fn parameters_mut(&mut self) -> &mut PropertyMap<String> {
        &mut self.parameters
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
        &self.movie.data()[self.start..self.end]
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

    /// Construct a new slice with a given dataset only.
    ///
    /// This is used primarily for converting owned data back into a slice: we
    /// reattach the SWF data to a fresh movie and return a new slice into it.
    pub fn owned_subslice(&self, data: Vec<u8>, source: &SwfMovie) -> Self {
        let len = data.len();

        Self {
            movie: Arc::new(self.movie.from_movie_and_subdata(data, source)),
            start: 0,
            end: len,
        }
    }

    /// Construct a new SwfSlice from a regular slice.
    ///
    /// This function returns None if the given slice is not a subslice of the
    /// current slice.
    pub fn to_subslice(&self, slice: &[u8]) -> Option<SwfSlice> {
        let self_pval = self.movie.data().as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Some(SwfSlice {
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
    pub fn resize_to_reader(&self, reader: &mut SwfStream<&[u8]>, size: usize) -> Option<SwfSlice> {
        if self.movie.data().as_ptr() as usize <= reader.get_ref().get_ref().as_ptr() as usize
            && (reader.get_ref().get_ref().as_ptr() as usize)
                < self.movie.data().as_ptr() as usize + self.movie.data().len()
        {
            let outer_offset =
                reader.get_ref().get_ref().as_ptr() as usize - self.movie.data().as_ptr() as usize;
            let inner_offset = reader.get_ref().position() as usize;
            let new_start = outer_offset + inner_offset;
            let new_end = outer_offset + inner_offset + size;

            let len = self.movie.data().len();

            if new_start < len && new_end < len {
                Some(SwfSlice {
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
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Option<SwfSlice> {
        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_start <= new_end {
            self.to_subslice(&self.movie.data().get(new_start..new_end)?)
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
        self.movie.header().version
    }

    /// Construct a reader for this slice.
    ///
    /// The `from` parameter is the offset to start reading the slice from.
    pub fn read_from(&self, from: u64) -> swf::read::Reader<std::io::Cursor<&[u8]>> {
        let mut cursor = std::io::Cursor::new(self.data());
        cursor.set_position(from);
        swf::read::Reader::new(cursor, self.movie.version())
    }
}

pub fn decode_tags<'a, R, F>(
    reader: &'a mut SwfStream<R>,
    mut tag_callback: F,
    stop_tag: TagCode,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: 'a + AsRef<[u8]>,
    F: FnMut(&mut SwfStream<R>, TagCode, usize) -> DecodeResult,
{
    use std::io::{Seek, SeekFrom};
    loop {
        let (tag_code, tag_len) = reader.read_tag_code_and_length()?;
        let end_pos = reader.get_ref().position() + tag_len as u64;

        let tag = TagCode::from_u16(tag_code);
        if let Some(tag) = tag {
            let result = tag_callback(reader, tag, tag_len);

            if let Err(e) = result {
                log::error!("Error running definition tag: {:?}, got {}", tag, e);
            }

            if stop_tag == tag {
                reader.get_mut().seek(SeekFrom::Start(end_pos))?;
                break;
            }
        } else {
            log::warn!("Unknown tag code: {:?}", tag_code);
        }

        reader.get_mut().seek(SeekFrom::Start(end_pos))?;
    }

    Ok(())
}
