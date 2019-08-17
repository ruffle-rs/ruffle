use swf::TagCode;

pub type DecodeResult = Result<(), Box<dyn std::error::Error>>;
pub type SwfStream<R> = swf::read::Reader<std::io::Cursor<R>>;

#[derive(Debug, Clone)]
pub struct SwfSlice {
    pub data: std::sync::Arc<Vec<u8>>,
    pub start: usize,
    pub end: usize,
}

impl AsRef<[u8]> for SwfSlice {
    fn as_ref(&self) -> &[u8] {
        &self.data[self.start..self.end]
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
                break;
            }
        } else {
            log::warn!("Unknown tag code: {:?}", tag_code);
        }

        use std::io::{Seek, SeekFrom};
        reader.get_mut().seek(SeekFrom::Start(end_pos))?;
    }

    Ok(())
}
