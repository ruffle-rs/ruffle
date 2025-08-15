use crate::error::Error;
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

    /// The tag data was recognized but could not be parsed due to an error.
    ///
    /// The error contained will never be EndOfData; this should only be used
    /// to flag unparsable data within an otherwise complete tag.
    Invalid(Error),
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
    /// Errors can be reported in one of two ways. If the header cannot be read
    /// then this function returns the error normally. However, if the header
    /// can be read, but the data inside the tag is corrupt, then a
    /// TagData::Invalid will be returned with the inner error. EndOfData will
    /// always be reported as a normal error and not as an invalid tag.
    ///
    /// In the event of an invalid header or end-of-data error, the reader
    /// position will be unchanged. Valid headers, with or without valid tag
    /// data, will seek the reader to the start of the next tag. This allows
    /// skipping past invalid tags.
    pub fn parse(reader: &mut FlvReader<'a>) -> Result<Self, Error> {
        let old_position = reader.stream_position()?;

        let ret = (|| {
            let _previous_tag_size = reader.read_u32()?;

            let tag_type = reader.read_u8()?;
            let data_size = reader.read_u24()?;
            let timestamp = reader.read_u24()?;
            let timestamp_extended = reader.read_u8()?;
            let stream_id = reader.read_u24()?;

            let timestamp = (((timestamp_extended as u32) << 24) | timestamp) as i32;
            let data_position = reader.stream_position()?;
            let new_position = data_position + data_size as u64;

            Ok((
                match tag_type {
                    8 => Tag {
                        timestamp,
                        stream_id,
                        data: match AudioData::parse(reader, data_size) {
                            Ok(data) => TagData::Audio(data),
                            Err(Error::EndOfData) => return Err(Error::EndOfData),
                            Err(e) => TagData::Invalid(e),
                        },
                    },
                    9 => Tag {
                        timestamp,
                        stream_id,
                        data: match VideoData::parse(reader, data_size) {
                            Ok(data) => TagData::Video(data),
                            Err(Error::EndOfData) => return Err(Error::EndOfData),
                            Err(e) => TagData::Invalid(e),
                        },
                    },
                    18 => Tag {
                        timestamp,
                        stream_id,
                        data: match ScriptData::parse(reader, data_size) {
                            Ok(data) => TagData::Script(data),
                            Err(Error::EndOfData) => return Err(Error::EndOfData),
                            Err(e) => TagData::Invalid(e),
                        },
                    },
                    unk => Tag {
                        timestamp,
                        stream_id,
                        data: TagData::Invalid(Error::UnknownTagType(unk)),
                    },
                },
                new_position,
            ))
        })();

        match ret {
            Ok((tag, new_position)) => {
                reader.seek(SeekFrom::Start(new_position))?;
                Ok(tag)
            }
            Err(e) => {
                reader.seek(SeekFrom::Start(old_position))?;
                Err(e)
            }
        }
    }

    /// Skip back to the prior tag in the FLV.
    ///
    /// FLV files are constructed as a list of tags. Back pointers to prior
    /// tags are provided to allow reverse seeking. This function ignores the
    /// tag at the current location and skips back to prior data in the file.
    pub fn skip_back(reader: &mut FlvReader<'a>) -> Result<(), Error> {
        let previous_tag_size = reader.read_u32()?;

        if previous_tag_size == 0 {
            // We have to stay in a valid end position but we can't seek back
            // any further, so we un-read the prior u32
            reader.seek(SeekFrom::Current(-4))?;
            Err(Error::EndOfData)
        } else {
            // NOTE: We need to seek back an extra 4 bytes for the u32 we just
            // read, which gets us to the start of the next tag. Then we need
            // to skip back another 4 bytes for the prior tag's back tag so
            // that we're in a valid position for another `parse`/`skip_back`.
            reader.seek(SeekFrom::Current(-(previous_tag_size as i64) - 8))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::reader::FlvReader;
    use crate::script::{ScriptData, Value, Variable};
    use crate::sound::{AudioData, AudioDataType, SoundFormat, SoundRate, SoundSize, SoundType};
    use crate::tag::{Tag, TagData};
    use crate::video::{CodecId, FrameType, VideoData, VideoPacket};

    #[test]
    fn read_tag_sounddata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0xBF, 0x12, 0x34, 0x56, 0x78,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Ok(Tag {
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
    fn read_tag_sounddata_invalid() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0xCA, 0x12, 0x34, 0x56, 0x78,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Ok(Tag {
                timestamp: 0,
                stream_id: 0x5000,
                data: TagData::Invalid(Error::UnknownAudioFormatType(0x0C))
            })
        )
    }

    #[test]
    fn read_tag_videodata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50,
            0x00, 0x12, 0x12, 0x34, 0x56, 0x78,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Ok(Tag {
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
            0x00, 0x02, 0x00, 0x03, 0x01, 0x02, 0x03, 0x06, 0x00, 0x03, 0x01, 0x02, 0x03, 0x05,
            0x00, 0x00, 0x09,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Ok(Tag {
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

    #[test]
    fn read_tag_onmetadata() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x0A, 0x6F, 0x6E, 0x4D, 0x65, 0x74, 0x61, 0x44, 0x61, 0x74, 0x61,
            0x08, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x08, 0x64, 0x75, 0x72, 0x61, 0x74, 0x69, 0x6F,
            0x6E, 0x00, 0x3F, 0xF1, 0x1E, 0xB8, 0x51, 0xEB, 0x85, 0x1F, 0x00, 0x05, 0x77, 0x69,
            0x64, 0x74, 0x68, 0x00, 0x40, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06,
            0x68, 0x65, 0x69, 0x67, 0x68, 0x74, 0x00, 0x40, 0x6D, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0D, 0x76, 0x69, 0x64, 0x65, 0x6F, 0x64, 0x61, 0x74, 0x61, 0x72, 0x61,
            0x74, 0x65, 0x00, 0x40, 0x68, 0x6A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x66,
            0x72, 0x61, 0x6D, 0x65, 0x72, 0x61, 0x74, 0x65, 0x00, 0x40, 0x28, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0C, 0x76, 0x69, 0x64, 0x65, 0x6F, 0x63, 0x6F, 0x64, 0x65,
            0x63, 0x69, 0x64, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D,
            0x61, 0x75, 0x64, 0x69, 0x6F, 0x64, 0x61, 0x74, 0x61, 0x72, 0x61, 0x74, 0x65, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x61, 0x75, 0x64, 0x69,
            0x6F, 0x73, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x72, 0x61, 0x74, 0x65, 0x00, 0x40, 0xE5,
            0x88, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x61, 0x75, 0x64, 0x69, 0x6F, 0x73,
            0x61, 0x6D, 0x70, 0x6C, 0x65, 0x73, 0x69, 0x7A, 0x65, 0x00, 0x40, 0x30, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x73, 0x74, 0x65, 0x72, 0x65, 0x6F, 0x01, 0x00,
            0x00, 0x0C, 0x61, 0x75, 0x64, 0x69, 0x6F, 0x63, 0x6F, 0x64, 0x65, 0x63, 0x69, 0x64,
            0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65, 0x6E, 0x63,
            0x6F, 0x64, 0x65, 0x72, 0x02, 0x00, 0x0D, 0x4C, 0x61, 0x76, 0x66, 0x35, 0x38, 0x2E,
            0x32, 0x31, 0x2E, 0x31, 0x30, 0x30, 0x00, 0x08, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x69,
            0x7A, 0x65, 0x00, 0x40, 0xCC, 0x73, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09,
        ];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Tag::parse(&mut reader),
            Ok(Tag {
                timestamp: 0,
                stream_id: 0,
                data: TagData::Script(ScriptData(vec![Variable {
                    name: b"onMetaData",
                    data: Value::EcmaArray(vec![
                        Variable {
                            name: b"duration",
                            data: Value::Number(1.07)
                        },
                        Variable {
                            name: b"width",
                            data: Value::Number(320.0)
                        },
                        Variable {
                            name: b"height",
                            data: Value::Number(233.0)
                        },
                        Variable {
                            name: b"videodatarate",
                            data: Value::Number(195.3125)
                        },
                        Variable {
                            name: b"framerate",
                            data: Value::Number(12.0)
                        },
                        Variable {
                            name: b"videocodecid",
                            data: Value::Number(2.0)
                        },
                        Variable {
                            name: b"audiodatarate",
                            data: Value::Number(0.0)
                        },
                        Variable {
                            name: b"audiosamplerate",
                            data: Value::Number(44100.0)
                        },
                        Variable {
                            name: b"audiosamplesize",
                            data: Value::Number(16.0)
                        },
                        Variable {
                            name: b"stereo",
                            data: Value::Boolean(false)
                        },
                        Variable {
                            name: b"audiocodecid",
                            data: Value::Number(2.0)
                        },
                        Variable {
                            name: b"encoder",
                            data: Value::String(b"Lavf58.21.100")
                        },
                        Variable {
                            name: b"filesize",
                            data: Value::Number(14567.0)
                        }
                    ])
                }]))
            })
        )
    }
}
