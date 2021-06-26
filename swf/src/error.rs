use crate::tag_code::TagCode;
use std::{borrow, error, fmt, io};

/// A `Result` from reading SWF data.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An error occurred while parsing an AVM1 action.
    /// This can contain sub-errors with further information (`Error::source`)
    Avm1ParseError {
        opcode: u8,
        source: Option<Box<dyn error::Error + 'static>>,
    },

    /// Invalid or unknown data was encountered.
    InvalidData(borrow::Cow<'static, str>),

    /// An error occurred while parsing an SWF tag.
    /// This can contain sub-errors with further information (`Error::source`)
    SwfParseError {
        tag_code: u16,
        source: Box<dyn error::Error + 'static>,
    },

    /// An IO error occurred (probably unexpected EOF).
    IoError(io::Error),

    /// This SWF requires unsupported features.
    Unsupported(borrow::Cow<'static, str>),
}

impl Error {
    /// Helper method to create `Error::Avm1ParseError`.
    #[inline]
    pub fn avm1_parse_error(opcode: u8) -> Self {
        Self::Avm1ParseError {
            opcode,
            source: None,
        }
    }

    /// Helper method to create `Error::Avm1ParseError`.
    #[inline]
    pub fn avm1_parse_error_with_source(opcode: u8, source: impl error::Error + 'static) -> Self {
        Self::Avm1ParseError {
            opcode,
            source: Some(Box::new(source)),
        }
    }

    /// Helper method to create `Error::InvalidData`.
    #[inline]
    pub fn invalid_data(message: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self::InvalidData(message.into())
    }

    /// Helper method to create `Error::SwfParseError`.
    #[inline]
    pub fn swf_parse_error(tag_code: u16, source: impl error::Error + 'static) -> Self {
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
        use crate::num_traits::FromPrimitive;
        match self {
            Self::Avm1ParseError { opcode, source } => {
                let op = crate::avm1::opcode::OpCode::from_u8(*opcode);
                "Error parsing AVM1 action ".fmt(f)?;
                if let Some(op) = op {
                    write!(f, "{:?}", op)?;
                } else {
                    write!(f, "Unknown({})", opcode)?;
                };
                if let Some(source) = source {
                    write!(f, ": {}", source)?;
                }
                Ok(())
            }
            Self::SwfParseError { tag_code, source } => {
                "Error parsing SWF tag ".fmt(f)?;
                if let Some(tag_code) = TagCode::from_u16(*tag_code) {
                    write!(f, "{:?}", tag_code)?;
                } else {
                    write!(f, "Unknown({})", tag_code)?;
                };
                write!(f, ": {}", source)?;
                Ok(())
            }
            Self::IoError(e) => e.fmt(f),
            Self::InvalidData(message) => write!(f, "Invalid data: {}", message),
            Self::Unsupported(message) => write!(f, "Unsupported data: {}", message),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use std::ops::Deref;
        match self {
            Self::Avm1ParseError { source, .. } => source.as_ref().map(|s| s.deref()),
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
