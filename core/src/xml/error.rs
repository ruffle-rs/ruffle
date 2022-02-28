//! Error types used in XML handling

use gc_arena::Collect;
use quick_xml::Error as QXError;
use std::error::Error as StdError;
use std::fmt::Error as FmtError;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid XML")]
    InvalidXml(#[from] ParseError),
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::InvalidXml(ParseError::from_quickxml_error(QXError::Utf8(
            error.utf8_error(),
        )))
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::InvalidXml(ParseError::from_quickxml_error(QXError::Utf8(error)))
    }
}

impl From<QXError> for Error {
    fn from(error: QXError) -> Self {
        Error::InvalidXml(ParseError::from_quickxml_error(error))
    }
}

/// Boxed `quick_xml` error
///
/// We can't clone `quick_xml` errors, nor can we clone several of the error
/// types it wraps over, so this creates an RC boxed version of the error that
/// can then be used elsewhere.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ParseError(Rc<QXError>);

impl ParseError {
    ///Convert a quick_xml error into a `ParseError`.
    pub fn from_quickxml_error(err: QXError) -> Self {
        ParseError(Rc::new(err))
    }

    pub fn ref_error(&self) -> &QXError {
        &*self.0
    }
}

impl Display for ParseError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        self.0.fmt(fmt)
    }
}

impl StdError for ParseError {
    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn StdError> {
        self.0.cause()
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
