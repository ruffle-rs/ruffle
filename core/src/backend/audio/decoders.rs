mod adpcm;
mod mp3;

pub use adpcm::AdpcmDecoder;
pub use mp3::Mp3Decoder;

pub trait Decoder: Iterator<Item = i16> {
    fn num_channels(&self) -> u8;
    fn sample_rate(&self) -> u16;
}

pub fn stream_tag_reader(
    swf_data: crate::tag_utils::SwfSlice,
) -> IterRead<impl Iterator<Item = u8>> {
    use std::io::{Cursor, Read};
    use swf::TagCode;

    let mut reader = swf::read::Reader::new(Cursor::new(swf_data), 8);
    let mut audio_data = vec![];
    let mut cur_byte = 0;
    let mut frame = 1;
    let iter = std::iter::from_fn(move || {
        if cur_byte >= audio_data.len() {
            cur_byte = 0;
            let tag_callback =
                |reader: &mut swf::read::Reader<Cursor<crate::tag_utils::SwfSlice>>,
                 tag_code,
                 tag_len| match tag_code {
                    TagCode::ShowFrame => {
                        frame += 1;
                        Ok(())
                    }
                    TagCode::SoundStreamBlock => {
                        audio_data.clear();
                        let mut data = vec![];
                        reader
                            .get_mut()
                            .take(tag_len as u64)
                            .read_to_end(&mut data)?;
                        audio_data.extend(data[4..].iter());
                        Ok(())
                    }
                    _ => Ok(()),
                };

            let _ =
                crate::tag_utils::decode_tags(&mut reader, tag_callback, TagCode::SoundStreamBlock);
        }

        if cur_byte < audio_data.len() {
            let byte = audio_data[cur_byte];
            cur_byte += 1;
            Some(byte)
        } else {
            None
        }
    });
    IterRead(iter)
}

pub struct IterRead<I: Iterator<Item = u8>>(I);

impl<I: Iterator<Item = u8>> std::io::Read for IterRead<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n = 0;
        for out in buf {
            if let Some(v) = self.0.next() {
                *out = v;
                n += 1;
            } else {
                break;
            }
        }
        Ok(n)
    }
}
