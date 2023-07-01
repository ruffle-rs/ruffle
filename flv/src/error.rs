use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
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

    #[error("IO error ({0}, {1})")]
    IoError(IoErrorKind, String),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::IoError(error.kind(), error.to_string())
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
            (Self::IoError(sk, ss), Self::IoError(ok, os)) => sk == ok && ss == os,
            _ => false,
        }
    }
}
