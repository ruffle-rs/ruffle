use crate::reader::FlvReader;
use crate::script::ScriptData;
use crate::sound::AudioData;
use crate::video::VideoData;

use std::io::{Seek, SeekFrom};

#[repr(u8)]
#[derive(PartialEq, Debug, Clone)]
pub enum TagData<'a> {
    Audio(AudioData<'a>) = 8,
    Video(VideoData<'a>) = 9,
    Script(ScriptData<'a>) = 18,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Tag<'a> {
    pub timestamp: i32,
    pub stream_id: u32, //24 bits max
    pub data: TagData<'a>,
}

impl<'a> Tag<'a> {
    /// Parse a single FLV tag structure.
    ///
    /// FLV files are constructed as a list of tags. Back pointers to prior
    /// tags are provided to allow reverse seeking. This function ignores the
    /// back pointer and parses the tag at the current location. At the end of
    /// parsing, the reader will be pointing to the start of the next tag. Thus,
    /// repeated calls to `parse` will yield further tags until the end of the
    /// file.
    ///
    /// `None` indicates that either:
    ///
    ///  * There is not enough data in the reader to read the next tag
    ///  * The data in the reader is corrupt and not a valid FLV
    ///
    /// If encountered, the position of the reader will be unchanged.
    pub fn parse(reader: &mut FlvReader<'a>) -> Option<Self> {
        let old_position = reader.stream_position().ok()?;

        let ret = (|| {
            let _previous_tag_size = reader.read_u32()?;

            let tag_type = reader.read_u8()?;
            let data_size = reader.read_u24()?;
            let timestamp = reader.read_u24()?;
            let timestamp_extended = reader.read_u8()?;
            let stream_id = reader.read_u24()?;

            let timestamp = ((timestamp_extended as u32) << 24 | timestamp) as i32;

            let new_position = reader.stream_position().ok()? + data_size as u64;

            Some((
                match tag_type {
                    8 => Tag {
                        timestamp,
                        stream_id,
                        data: TagData::Audio(AudioData::parse(reader, data_size)?),
                    },
                    9 => Tag {
                        timestamp,
                        stream_id,
                        data: TagData::Video(VideoData::parse(reader, data_size)?),
                    },
                    18 => Tag {
                        timestamp,
                        stream_id,
                        data: TagData::Script(ScriptData::parse(reader)?),
                    },
                    _ => return None,
                },
                new_position,
            ))
        })();

        if let Some((tag, new_position)) = ret {
            reader.seek(SeekFrom::Start(new_position)).ok()?;
            Some(tag)
        } else {
            reader.seek(SeekFrom::Start(old_position)).ok()?;
            None
        }
    }

    /// Skip back to the prior tag in the FLV.
    ///
    /// FLV files are constructed as a list of tags. Back pointers to prior
    /// tags are provided to allow reverse seeking. This function ignores the
    /// tag at the current location and skips back to prior data in the file.
    pub fn skip_back(reader: &mut FlvReader<'a>) -> Option<()> {
        let previous_tag_size = reader.read_u32()?;
        reader
            .seek(SeekFrom::Current(-(previous_tag_size as i64)))
            .ok()?;

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::FlvReader;
    use crate::script::{ScriptData, Value, Variable};
    use crate::sound::{AudioData, AudioDataType, SoundFormat, SoundRate, SoundSize, SoundType};
    use crate::tag::{Tag, TagData};
    use crate::video::{CodecId, FrameType, VideoData, VideoPacket};

    #[test]
    fn read_tag_sounddata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0xFB, 0x12, 0x34, 0x56, 0x78,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Some(Tag {
                timestamp: 0,
                stream_id: 0x5000,
                data: TagData::Audio(AudioData {
                    format: SoundFormat::Speex,
                    rate: SoundRate::R44_000,
                    size: SoundSize::Bits16,
                    sound_type: SoundType::Stereo,
                    data: AudioDataType::Raw(&[0x12, 0x34, 0x56, 0x78])
                })
            })
        )
    }

    #[test]
    fn read_tag_videodata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0x21, 0x12, 0x34, 0x56, 0x78,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Some(Tag {
                timestamp: 0,
                stream_id: 0x5000,
                data: TagData::Video(VideoData {
                    frame_type: FrameType::Keyframe,
                    codec_id: CodecId::SorensonH263,
                    data: VideoPacket::Data(&[0x12, 0x34, 0x56, 0x78])
                })
            })
        )
    }

    #[test]
    fn read_tag_scriptdata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0x02, 0x00, 0x03, 0x01, 0x02, 0x03, 0x06, 0x00, 0x03, 0x01, 0x02, 0x03, 0x05, 0x00,
            0x00, 0x09,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Some(Tag {
                timestamp: 0,
                stream_id: 0x5000,
                data: TagData::Script(ScriptData(vec![
                    Variable {
                        name: &[0x01, 0x02, 0x03],
                        data: Value::Undefined
                    },
                    Variable {
                        name: &[0x01, 0x02, 0x03],
                        data: Value::Null
                    }
                ]))
            })
        )
    }
}
