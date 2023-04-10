use crate::reader::FlvReader;
use crate::script::ScriptData;
use crate::sound::AudioData;
use crate::video::VideoData;

#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum TagData<'a> {
    Audio(AudioData<'a>) = 8,
    Video(VideoData<'a>) = 9,
    Script(ScriptData<'a>) = 18,
}

#[derive(PartialEq, Debug)]
pub struct Tag<'a> {
    timestamp: i32,
    stream_id: u32, //24 bits max
    data: TagData<'a>,
}

impl<'a> Tag<'a> {
    /// Parse a single FLV tag structure.
    ///
    /// This assumes the reader is currently pointing at the tag itself, not
    /// the back-pointers in between the tags.
    pub fn parse(reader: &mut FlvReader<'a>) -> Option<Self> {
        let tag_type = reader.read_u8()?;
        let data_size = reader.read_u24()?;
        let timestamp = reader.read_u24()?;
        let timestamp_extended = reader.read_u8()?;
        let stream_id = reader.read_u24()?;

        let timestamp = ((timestamp_extended as u32) << 24 | timestamp) as i32;

        match tag_type {
            8 => Some(Tag {
                timestamp,
                stream_id,
                data: TagData::Audio(AudioData::parse(reader, data_size)?),
            }),
            9 => Some(Tag {
                timestamp,
                stream_id,
                data: TagData::Video(VideoData::parse(reader, data_size)?),
            }),
            18 => Some(Tag {
                timestamp,
                stream_id,
                data: TagData::Script(ScriptData::parse(reader)?),
            }),
            _ => None,
        }
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
            0x08, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0xFB, 0x12, 0x34,
            0x56, 0x78,
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
            0x09, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x21, 0x12, 0x34,
            0x56, 0x78,
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
            0x12, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x03, 0x01,
            0x02, 0x03, 0x06, 0x00, 0x03, 0x01, 0x02, 0x03, 0x05, 0x00, 0x00, 0x09,
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
