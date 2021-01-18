//! String typed used by SWF files.
//!
//! Allows for locale-dependent encoding for SWF version <6.

use encoding_rs::{Encoding, UTF_8};
use std::{borrow::Cow, fmt};

/// `SwfStr` is returned by SWF and AVM1 parsing functions.
/// `SwfStr` is analogous to `&str`, with some additional allowances:
/// * An encoding is specified along with the string data.
/// * The string contains no null bytes.
/// * Invalid data for any particular encoding is allowed;
///   any conversions to std::String will be lossy for invalid data.
/// This handles the locale dependent encoding of early SWF files and
/// mimics C-style null-terminated string behavior.
/// To convert this to a standard Rust string, use `SwfStr::to_str_lossy`.
/// `SwfStr`s are equal if both their encoding and data matches.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SwfStr<'a> {
    /// The string bytes.
    string: &'a [u8],

    /// The encoding of the string data.
    encoding: &'static Encoding,
}

impl<'a> SwfStr<'a> {
    /// Create a new `SwfStr` from a byte slice with a given encoding.
    /// The string will be truncated if a null byte is encountered.
    /// The data is not required to be valid for the given encoding.
    #[inline]
    pub fn from_bytes(string: &'a [u8], encoding: &'static Encoding) -> Self {
        let i = string
            .into_iter()
            .position(|&c| c == 0)
            .unwrap_or(string.len());
        Self {
            string: &string[..i],
            encoding,
        }
    }

    /// Create a new `SwfStr` from a byte slice with a given encoding.
    /// The string should contain no null bytes, but ths is not checked.
    /// The data is not required to be valid for the given encoding.
    #[inline]
    pub unsafe fn from_bytes_unchecked(string: &'a [u8], encoding: &'static Encoding) -> Self {
        Self { string, encoding }
    }

    /// Create a new UTF-8 `SwfStr` from a Rust `str`.
    /// The string will be truncated if a null byte is encountered.
    #[inline]
    pub fn from_str(string: &'a str) -> Self {
        Self::from_bytes(string.as_bytes(), UTF_8)
    }

    /// Create a new `SwfStr` with the given encoding from a Rust `str`.
    /// The string will be re-encoded with the given encoding.
    /// The string will be truncated if a null byte is encountered.
    /// `None` is returned if the encoding is not lossless.
    /// Intended for tests.
    pub fn from_str_with_encoding(string: &'a str, encoding: &'static Encoding) -> Option<Self> {
        if let (Cow::Borrowed(s), _, false) = encoding.encode(&string) {
            Some(Self::from_bytes(s, encoding))
        } else {
            None
        }
    }

    /// Returns the byte slice of this string.
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.string
    }

    /// Returns the encoding used by this string.
    #[inline]
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// Returns `true` if the string has a length of zero, and `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    /// Returns the `len` of the string in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Decodes the string into a Rust UTF-8 `str`.
    /// The UTF-8 replacement character will be uses for any invalid data.
    #[inline]
    pub fn to_str_lossy(&self) -> Cow<'a, str> {
        self.encoding.decode_without_bom_handling(self.string).0
    }

    /// Decodes the string into a Rust UTF-8 `String`.
    /// The UTF-8 replacement character will be uses for any invalid data.
    #[inline]
    pub fn to_string_lossy(&self) -> String {
        self.to_str_lossy().into_owned()
    }
}

impl<'a> Default for SwfStr<'a> {
    fn default() -> Self {
        Self {
            string: &[],
            encoding: UTF_8,
        }
    }
}

impl<'a> From<&'a str> for SwfStr<'a> {
    fn from(s: &'a str) -> Self {
        SwfStr::from_str(s)
    }
}

impl<'a, T: AsRef<str>> PartialEq<T> for SwfStr<'a> {
    fn eq(&self, other: &T) -> bool {
        self.string == other.as_ref().as_bytes()
    }
}

impl<'a> fmt::Display for SwfStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str_lossy())
    }
}

impl<'a> fmt::Debug for SwfStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str_lossy())
    }
}
