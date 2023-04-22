use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the FLV parser ran out of data")]
    EndOfData,

    #[error("the FLV cannot be read as its length exceeds the maximum memory size for this architecture")]
    PointerTooBig,

    #[error("the data stream does not have a valid FLV header signature")]
    WrongMagic,

    #[error("the FLV contains a script data block with a value of unknown type")]
    UnknownValueType,

    #[error("the FLV contains an audio data block that is too short")]
    ShortAudioBlock,

    #[error("the FLV contains an audio data block with unknown format type {0}")]
    UnknownAudioFormatType(u8),

    #[error("the FLV contains an audio data block with unknown sample rate {0}")]
    UnknownAudioRate(u8),

    #[error("the FLV contains an audio data block with unknown sample size {0}")]
    UnknownAudioSampleSize(u8),

    #[error("the FLV contains an audio data block with unknown channel count {0}")]
    UnknownAudioChannelCount(u8),

    #[error("the FLV contains an audio data block with AAC data that is of unknown type {0}")]
    UnknownAacPacketType(u8),

    #[error("the FLV contains a video data block that is too short")]
    ShortVideoBlock,

    #[error("the FLV contains a video data block with unknown frame type {0}")]
    UnknownVideoFrameType(u8),

    #[error("the FLV contains a video data block with unknown codec {0}")]
    UnknownVideoCodec(u8),

    #[error("the FLV contains a video data block with a command frame of unknown type {0}")]
    UnknownVideoCommandType(u8),

    #[error("the FLV contains a video data block with AVC data that is of unknown type {0}")]
    UnknownAvcPacketType(u8),

    #[error("the FLV contains a tag with unknown type {0}")]
    UnknownTagType(u8),

    #[error("IO error ({0})")]
    IoError(IoError),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::IoError(error)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Self::EndOfData, Self::EndOfData) => true,
            (Self::PointerTooBig, Self::PointerTooBig) => true,
            (Self::WrongMagic, Self::WrongMagic) => true,
            (Self::UnknownValueType, Self::UnknownValueType) => true,
            (Self::ShortAudioBlock, Self::ShortAudioBlock) => true,
            (Self::UnknownAudioFormatType(s), Self::UnknownAudioFormatType(o)) => s == o,
            (Self::UnknownAudioRate(s), Self::UnknownAudioRate(o)) => s == o,
            (Self::UnknownAudioSampleSize(s), Self::UnknownAudioSampleSize(o)) => s == o,
            (Self::UnknownAudioChannelCount(s), Self::UnknownAudioChannelCount(o)) => s == o,
            (Self::UnknownAacPacketType(s), Self::UnknownAacPacketType(o)) => s == o,
            (Self::ShortVideoBlock, Self::ShortVideoBlock) => true,
            (Self::UnknownVideoFrameType(s), Self::UnknownVideoFrameType(o)) => s == o,
            (Self::UnknownVideoCodec(s), Self::UnknownVideoCodec(o)) => s == o,
            (Self::UnknownVideoCommandType(s), Self::UnknownVideoCommandType(o)) => s == o,
            (Self::UnknownAvcPacketType(s), Self::UnknownAvcPacketType(o)) => s == o,
            (Self::UnknownTagType(s), Self::UnknownTagType(o)) => s == o,
            (Self::IoError(s), Self::IoError(o)) => s.kind() == o.kind(),
            _ => false,
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::EndOfData => Self::EndOfData,
            Self::PointerTooBig => Self::PointerTooBig,
            Self::WrongMagic => Self::WrongMagic,
            Self::UnknownValueType => Self::UnknownValueType,
            Self::ShortAudioBlock => Self::ShortAudioBlock,
            Self::UnknownAudioFormatType(unk) => Self::UnknownAudioFormatType(*unk),
            Self::UnknownAudioRate(unk) => Self::UnknownAudioRate(*unk),
            Self::UnknownAudioSampleSize(unk) => Self::UnknownAudioSampleSize(*unk),
            Self::UnknownAudioChannelCount(unk) => Self::UnknownAudioChannelCount(*unk),
            Self::UnknownAacPacketType(unk) => Self::UnknownAacPacketType(*unk),
            Self::ShortVideoBlock => Self::ShortVideoBlock,
            Self::UnknownVideoFrameType(unk) => Self::UnknownVideoFrameType(*unk),
            Self::UnknownVideoCodec(unk) => Self::UnknownVideoCodec(*unk),
            Self::UnknownVideoCommandType(unk) => Self::UnknownVideoCommandType(*unk),
            Self::UnknownAvcPacketType(unk) => Self::UnknownAvcPacketType(*unk),
            Self::UnknownTagType(unk) => Self::UnknownTagType(*unk),

            // IOError cannot be cloned, since you can attach arbitrary error
            // types to it. Instead, cloned FLV errors contain only the error
            // kind and text, which can be cloned.
            Self::IoError(ioe) => Self::IoError(IoError::new(ioe.kind(), ioe.to_string())),
        }
    }
}
