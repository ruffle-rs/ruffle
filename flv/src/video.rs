use crate::error::Error;
use crate::reader::FlvReader;
use std::io::Seek;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum FrameType {
    Keyframe = 1,
    Interframe = 2,
    InterframeDisposable = 3,
    Generated = 4,
    CommandFrame = 5,
}

impl TryFrom<u8> for FrameType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Keyframe),
            2 => Ok(Self::Interframe),
            3 => Ok(Self::InterframeDisposable),
            4 => Ok(Self::Generated),
            5 => Ok(Self::CommandFrame),
            unk => Err(Error::UnknownVideoFrameType(unk)),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum CodecId {
    Jpeg = 1,
    SorensonH263 = 2,
    ScreenVideo = 3,
    On2Vp6 = 4,
    On2Vp6Alpha = 5,
    ScreenVideo2 = 6,
    Avc = 7,
}

impl TryFrom<u8> for CodecId {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Jpeg),
            2 => Ok(Self::SorensonH263),
            3 => Ok(Self::ScreenVideo),
            4 => Ok(Self::On2Vp6),
            5 => Ok(Self::On2Vp6Alpha),
            6 => Ok(Self::ScreenVideo2),
            7 => Ok(Self::Avc),
            unk => Err(Error::UnknownVideoCodec(unk)),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum CommandFrame {
    StartOfClientSideSeek = 0,
    EndOfClientSideSeek = 1,
}

impl TryFrom<u8> for CommandFrame {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::StartOfClientSideSeek),
            1 => Ok(Self::EndOfClientSideSeek),
            unk => Err(Error::UnknownVideoCommandType(unk)),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VideoPacket<'a> {
    Data(&'a [u8]),
    AvcSequenceHeader(&'a [u8]),
    AvcNalu {
        composition_time_offset: i32,
        data: &'a [u8],
    },
    AvcEndOfSequence,
    CommandFrame(CommandFrame),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VideoData<'a> {
    pub frame_type: FrameType,
    pub codec_id: CodecId,
    pub data: VideoPacket<'a>,
}

impl<'a> VideoData<'a> {
    /// Parse a video data structure.
    ///
    /// This does not parse the actual video data itself, which is instead
    /// returned as an array that must be provided to your video decoder.
    ///
    /// `data_size` is the size of the entire video data structure, *including*
    /// the header. Errors are yielded if the `data_size` is too small for the
    /// video data present in the tag. This should not be confused for
    /// `EndOfData` which indicates that we've read past the end of the whole
    /// data stream.
    pub fn parse(reader: &mut FlvReader<'a>, data_size: u32) -> Result<Self, Error> {
        let start = reader.stream_position().expect("current position") as usize;
        let format_spec = reader.read_u8()?;

        let frame_type = FrameType::try_from(format_spec >> 4)?;
        let codec_id = CodecId::try_from(format_spec & 0x0F)?;

        let header_size = reader.stream_position().expect("current position") as usize - start;
        if (data_size as usize) < header_size {
            return Err(Error::ShortVideoBlock);
        }
        let data = reader.read(data_size as usize - header_size)?;

        let packet = match (frame_type, codec_id) {
            (FrameType::CommandFrame, _) => VideoPacket::CommandFrame(CommandFrame::try_from(
                *data.first().ok_or(Error::ShortVideoBlock)?,
            )?),
            (_, CodecId::Avc) => {
                let bytes = data.get(1..4).ok_or(Error::ShortVideoBlock)?;
                let is_negative = bytes[0] & 0x80 != 0;
                let composition_time_offset = i32::from_be_bytes([
                    if is_negative { 0xFF } else { 0x00 },
                    bytes[0],
                    bytes[1],
                    bytes[2],
                ]);

                match *data.first().ok_or(Error::ShortVideoBlock)? {
                    0 => VideoPacket::AvcSequenceHeader(&data[4..]),
                    1 => VideoPacket::AvcNalu {
                        composition_time_offset,
                        data: &data[4..],
                    },
                    2 => VideoPacket::AvcEndOfSequence,
                    unk => return Err(Error::UnknownAvcPacketType(unk)),
                }
            }
            (_, _) => VideoPacket::Data(data),
        };

        Ok(VideoData {
            frame_type,
            codec_id,
            data: packet,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::reader::FlvReader;
    use crate::video::{CodecId, FrameType, VideoData, VideoPacket};

    #[test]
    fn read_videodata() {
        let data = [0x12, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::SorensonH263,
                data: VideoPacket::Data(&[0x12, 0x34, 0x56, 0x78])
            })
        );
    }

    #[test]
    fn read_videodata_invalid_len() {
        let data = [0x12, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, 0),
            Err(Error::ShortVideoBlock)
        );
    }

    #[test]
    fn read_videodata_short_len() {
        let data = [0x12, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, 2),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::SorensonH263,
                data: VideoPacket::Data(&[0x12])
            })
        );
    }

    #[test]
    fn read_videodata_avcsequence() {
        let data = [0x17, 0x00, 0x00, 0x50, 0x00, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::Avc,
                data: VideoPacket::AvcSequenceHeader(&[0x12, 0x34, 0x56, 0x78])
            })
        );
    }

    #[test]
    fn read_videodata_avcnalu() {
        let data = [0x17, 0x01, 0x00, 0x50, 0x00, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::Avc,
                data: VideoPacket::AvcNalu {
                    composition_time_offset: 0x5000,
                    data: &[0x12, 0x34, 0x56, 0x78]
                }
            })
        );
    }

    #[test]
    fn read_videodata_avcnalu_negative() {
        let data = [0x17, 0x01, 0xFF, 0xFF, 0xFE, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::Avc,
                data: VideoPacket::AvcNalu {
                    composition_time_offset: -2,
                    data: &[0x12, 0x34, 0x56, 0x78]
                }
            })
        );
    }

    #[test]
    fn read_videodata_avceos() {
        let data = [0x17, 0x02, 0xFF, 0xFF, 0xFE, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Ok(VideoData {
                frame_type: FrameType::Keyframe,
                codec_id: CodecId::Avc,
                data: VideoPacket::AvcEndOfSequence
            })
        );
    }

    #[test]
    fn read_videodata_avcinvalid() {
        let data = [0x17, 0xFF, 0xFF, 0xFF, 0xFE, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            VideoData::parse(&mut reader, data.len() as u32),
            Err(Error::UnknownAvcPacketType(0xFF))
        );
    }
}
