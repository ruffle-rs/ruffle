//! String type used by SWF files.

pub use encoding_rs::{Encoding, SHIFT_JIS, UTF_8, WINDOWS_1252};
use std::{borrow::Cow, fmt};

/// `SwfStr` is the string type returned by SWF parsing functions.
/// `SwfStr` is a bstr-like type analogous to `str`:
/// * The encoding depends on the SWF version (UTF-8 for SWF6 and higher).
///   Use `Reader::encoding` or `SwfStr::encoding_for_version` to get the
///   proper encoding.
/// * Invalid data for any particular encoding is allowed;
///   any conversions to std::String will be lossy for invalid data.
/// To convert this to a standard Rust string, use `SwfStr::to_str_lossy`.
#[derive(Eq, PartialEq)]
#[repr(transparent)]
pub struct SwfStr {
    /// The string bytes.
    string: [u8],
}

impl SwfStr {
    /// Create a new `SwfStr` from a byte slice with a given encoding.
    /// The string will be truncated if a null byte is encountered.
    /// The data is not required to be valid for the given encoding.
    #[inline]
    pub fn from_bytes(string: &[u8]) -> &Self {
        unsafe { &*(string as *const [u8] as *const Self) }
    }

    #[inline]
    pub fn from_bytes_null_terminated(string: &[u8]) -> &Self {
        let i = string.iter().position(|&c| c == 0).unwrap_or(string.len());
        Self::from_bytes(&string[..i])
    }

    /// Create a new UTF-8 `SwfStr` from a Rust `str`.
    #[inline]
    pub fn from_utf8_str(string: &str) -> &Self {
        Self::from_bytes(string.as_bytes())
    }

    /// Create a new UTF-8 `SwfStr` from a Rust `str`.
    /// The string will be truncated if a null byte is encountered.
    #[inline]
    pub fn from_utf8_str_null_terminated(string: &str) -> &Self {
        Self::from_bytes_null_terminated(string.as_bytes())
    }

    /// Create a new `SwfStr` with the given encoding from a Rust `str`.
    /// The string will be re-encoded with the given encoding.
    /// The string will be truncated if a null byte is encountered.
    /// `None` is returned if the encoding is not lossless.
    /// Intended for tests.
    pub fn from_str_with_encoding<'a>(
        string: &'a str,
        encoding: &'static Encoding,
    ) -> Option<&'a Self> {
        if let (Cow::Borrowed(s), _, false) = encoding.encode(&string) {
            Some(Self::from_bytes(s))
        } else {
            None
        }
    }

    /// Returns the suggested string encoding for the given SWF version.
    /// For SWF version 6 and higher, this is always UTF-8.
    /// For SWF version 5 and lower, this is locale-dependent,
    /// and we default to WINDOWS-1252.
    #[inline]
    pub fn encoding_for_version(swf_version: u8) -> &'static Encoding {
        if swf_version >= 6 {
            UTF_8
        } else {
            WINDOWS_1252
        }
    }

    /// Returns the byte slice of this string.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.string
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
    pub fn to_str_lossy(&self, encoding: &'static Encoding) -> Cow<'_, str> {
        encoding.decode_without_bom_handling(&self.string).0
    }

    /// Decodes the string into a Rust UTF-8 `String`.
    /// The UTF-8 replacement character will be uses for any invalid data.
    #[inline]
    pub fn to_string_lossy(&self, encoding: &'static Encoding) -> String {
        self.to_str_lossy(encoding).into_owned()
    }
}

impl<'a> Default for &'a SwfStr {
    fn default() -> &'a SwfStr {
        SwfStr::from_bytes(&[])
    }
}

impl<'a> From<&'a str> for &'a SwfStr {
    fn from(s: &'a str) -> &'a SwfStr {
        SwfStr::from_utf8_str(s)
    }
}

impl<'a, T: ?Sized + AsRef<str>> PartialEq<T> for SwfStr {
    fn eq(&self, other: &T) -> bool {
        &self.string == other.as_ref().as_bytes()
    }
}

impl fmt::Debug for SwfStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note that this assumes UTF-8 encoding;
        // other encodings like Shift-JIS will output gibberish.
        f.write_str(&self.to_str_lossy(UTF_8))
    }
}
