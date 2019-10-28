use std::sync::Arc;
use swf::TagCode;

pub type DecodeResult = Result<(), Box<dyn std::error::Error>>;
pub type SwfStream<R> = swf::read::Reader<std::io::Cursor<R>>;

/// A shared-ownership reference to some portion of an immutable datastream.
#[derive(Debug, Clone)]
pub struct SwfSlice {
    pub data: Arc<Vec<u8>>,
    pub start: usize,
    pub end: usize,
}

impl AsRef<[u8]> for SwfSlice {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.data[self.start..self.end]
    }
}

impl SwfSlice {
    /// Creates an empty SwfSlice.
    #[inline]
    pub fn empty() -> Self {
        Self {
            data: Arc::new(vec![]),
            start: 0,
            end: 0,
        }
    }
    /// Construct a new SwfSlice from a regular slice.
    ///
    /// This function returns None if the given slice is not a subslice of the
    /// current slice.
    pub fn to_subslice(&self, slice: &[u8]) -> Option<SwfSlice> {
        let self_pval = self.data.as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Some(SwfSlice {
                data: self.data.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            })
        } else {
            None
        }
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

            if let Err(_e) = result {
                log::error!("Error running definition tag: {:?}", tag);
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
