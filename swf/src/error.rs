use crate::avm1::opcode::OpCode;
use crate::avm2::types as avm2;
use crate::tag_code::TagCode;
use std::{borrow, error, fmt, io};

/// A `Result` from reading SWF data.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    /// An error occurred while parsing an AVM1 action.
    /// This can contain sub-errors with further information (`Error::source`)
    Avm1ParseError(Avm1ParseError),

    // An error occurred while parsing ABC.
    AbcParseError(AbcParseError),

    /// Invalid or unknown data was encountered.
    InvalidData(borrow::Cow<'static, str>),

    /// An error occurred while parsing an SWF tag.
    /// This can contain sub-errors with further information (`Error::source`)
    SwfParseError {
        tag_code: u16,
        source: Box<dyn error::Error + Send + Sync + 'static>,
    },

    /// An IO error occurred (probably unexpected EOF).
    IoError(io::Error),

    /// This SWF requires unsupported features.
    Unsupported(borrow::Cow<'static, str>),
}

impl Error {
    /// Helper method to create `Error::InvalidData`.
    #[inline]
    pub fn invalid_data(message: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self::InvalidData(message.into())
    }

    /// Helper method to create `Error::SwfParseError`.
    #[inline]
    pub fn swf_parse_error(
        tag_code: u16,
        source: impl error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::SwfParseError {
            tag_code,
            source: Box::new(source),
        }
    }

    /// Helper method to create `Error::Unsupported`.
    #[inline]
    pub fn unsupported(message: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self::Unsupported(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Avm1ParseError(error) => {
                write!(f, "Error parsing AVM1 bytecode: {}", error)
            }
            Self::AbcParseError(error) => {
                write!(f, "Error parsing ABC: {}", error)
            }
            Self::SwfParseError { tag_code, source } => {
                write!(
                    f,
                    "Error parsing SWF tag {}: {}",
                    TagCode::format(*tag_code),
                    source
                )
            }
            Self::IoError(e) => e.fmt(f),
            Self::InvalidData(message) => write!(f, "Invalid data: {message}"),
            Self::Unsupported(message) => write!(f, "Unsupported data: {message}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Avm1ParseError(e) => e.source(),
            Self::AbcParseError(e) => e.source(),
            Self::IoError(e) => e.source(),
            Self::InvalidData(_) => None,
            Self::SwfParseError { source, .. } => Some(source.as_ref()),
            Self::Unsupported(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

#[derive(Debug)]
pub struct Avm1ParseError {
    // opcodes < 0x80 always have zero length, so we use 0 to mean 'incomplete action header'.
    pub(crate) opcode: u8,
    pub(crate) source: UnexpectedEof,
}

impl From<Avm1ParseError> for Error {
    fn from(e: Avm1ParseError) -> Error {
        Error::Avm1ParseError(e)
    }
}

impl Avm1ParseError {
    pub(crate) fn new(opcode: Option<u8>, source: UnexpectedEof) -> Self {
        let opcode = opcode.unwrap_or(0);
        Self { opcode, source }
    }
}

impl fmt::Display for Avm1ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.opcode {
            op @ 0x80.. => write!(f, "unterminated action {}", OpCode::format(op)),
            _ => write!(f, "incomplete action header"),
        }
    }
}

impl std::error::Error for Avm1ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum AbcParseError {
    MethodInfoOutOfBounds {
        method_count: u32,
        method_index: avm2::Index<avm2::Method>,
    },
    IllegalOpcode {
        opcode: u8,
    },
}

impl fmt::Display for AbcParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AbcParseError::MethodInfoOutOfBounds {
                method_count,
                method_index: avm2::Index(index, _),
            } => write!(
                f,
                "Method body refers to index {index} but there are only {method_count} method infos"
            ),
            AbcParseError::IllegalOpcode { opcode } => write!(f, "Illegal opcode {opcode:#x}"),
        }
    }
}

impl error::Error for AbcParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AbcParseError::MethodInfoOutOfBounds { .. } => None,
            AbcParseError::IllegalOpcode { .. } => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnexpectedEof(pub(crate) ());

impl From<UnexpectedEof> for Error {
    fn from(_: UnexpectedEof) -> Error {
        Error::IoError(io::ErrorKind::UnexpectedEof.into())
    }
}

impl fmt::Display for UnexpectedEof {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("unexpected end of SWF or tag")
    }
}

impl error::Error for UnexpectedEof {}

#[cfg(test)]
#[test]
fn test_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>()
}
