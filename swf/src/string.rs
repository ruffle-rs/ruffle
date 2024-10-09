//! String type used by SWF files.

use serde::Serialize;

pub use encoding_rs::{Encoding, SHIFT_JIS, UTF_8, WINDOWS_1252};
use std::{borrow::Cow, fmt};

/// A bstr-like string type analogous to [`str`] that's returned by SWF parsing functions:
///
/// * The encoding depends on the SWF version (UTF-8 for SWF6 and higher).
///   Use `Reader::encoding` or [`SwfStr::encoding_for_version`] to get the
///   proper encoding.
/// * Invalid data for any particular encoding is allowed;
///   any conversions to std::String will be lossy for invalid data.
///
/// To convert this to a standard Rust string, use [`SwfStr::to_str_lossy`].
#[derive(Eq, PartialEq, Serialize)]
#[repr(transparent)]
pub struct SwfStr {
    /// The string bytes.
    string: [u8],
}

impl SwfStr {
    /// Creates a new `SwfStr` from a byte slice.
    /// The data is not required to be valid for the given encoding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_bytes(b"Hello, World!");
    /// ```
    #[inline]
    pub const fn from_bytes(string: &[u8]) -> &Self {
        // SAFETY: Casting is safe because internal representations are
        // the same, see repr(transparent).
        unsafe { &*(string as *const [u8] as *const Self) }
    }

    /// Creates a `SwfStr` from a byte slice by reading until a NULL byte (`0`) is encountered.
    /// Returns `None` if no NULL byte was found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_bytes_null_terminated(b"I'm null-terminated!\0");
    /// assert!(s.is_some());
    ///
    /// let s = SwfStr::from_bytes_null_terminated(b"I'm not terminated...");
    /// assert!(s.is_none());
    /// ```
    #[inline]
    pub fn from_bytes_null_terminated(string: &[u8]) -> Option<&Self> {
        // If investigations show that the bounds check is not elided,
        // it should be safe to use `get_unchecked` here instead.
        // Initial Godbolt research shows it doesn't make a difference.
        string
            .iter()
            .position(|&c| c == 0)
            .map(|i| Self::from_bytes(&string[..i]))
    }

    /// Creates a new UTF-8 `SwfStr` from a Rust [`str`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_utf8_str("Hello, 🌏!");
    /// ```
    #[inline]
    pub const fn from_utf8_str(string: &str) -> &Self {
        Self::from_bytes(string.as_bytes())
    }

    /// Creates a new UTF-8 `SwfStr` from a Rust [`str`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_utf8_str_null_terminated("I'm null-terminated!\0");
    /// assert!(s.is_some());
    ///
    /// let s = SwfStr::from_utf8_str_null_terminated("I'm not terminated...");
    /// assert!(s.is_none());
    /// ```
    #[inline]
    pub fn from_utf8_str_null_terminated(string: &str) -> Option<&Self> {
        Self::from_bytes_null_terminated(string.as_bytes())
    }

    /// Creates a new `SwfStr` with the given encoding from a Rust [`str`].
    /// Returns `None` if the encoding is not lossless.
    ///
    /// The string will be re-encoded with the given encoding.
    /// The string will be truncated if a NULL byte (`0`) is encountered.
    ///
    /// Intended for tests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    /// use encoding_rs::WINDOWS_1252;
    ///
    /// let s = SwfStr::from_str_with_encoding("Hello, World!", WINDOWS_1252);
    /// assert!(s.is_some());
    /// ```
    pub fn from_str_with_encoding<'a>(
        string: &'a str,
        encoding: &'static Encoding,
    ) -> Option<&'a Self> {
        if let (Cow::Borrowed(s), _, false) = encoding.encode(string) {
            Some(Self::from_bytes(s))
        } else {
            None
        }
    }

    /// Returns the suggested string encoding for the given SWF version.
    ///
    /// For SWF version 6 and higher, this is always UTF-8.
    /// For SWF version 5 and lower, this is locale-dependent,
    /// and we default to WINDOWS-1252.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    /// use encoding_rs::{UTF_8, WINDOWS_1252};
    ///
    /// assert_eq!(SwfStr::encoding_for_version(9), UTF_8);
    /// assert_eq!(SwfStr::encoding_for_version(3), WINDOWS_1252);
    /// ```
    #[inline]
    pub fn encoding_for_version(swf_version: u8) -> &'static Encoding {
        if swf_version >= 6 {
            UTF_8
        } else {
            WINDOWS_1252
        }
    }

    /// Returns the byte slice of this string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_utf8_str("💖");
    /// assert_eq!(s.as_bytes(), [0xF0, 0x9F, 0x92, 0x96]);
    /// ```
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.string
    }

    /// Returns `true` if the string has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_bytes(&[]);
    /// assert!(s.is_empty());
    ///
    /// let s = SwfStr::from_utf8_str("💖");
    /// assert!(!s.is_empty());
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    /// Returns the length of the string in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    ///
    /// let s = SwfStr::from_utf8_str("");
    /// assert_eq!(s.len(), 0);
    ///
    /// let s = SwfStr::from_utf8_str("Hi!");
    /// assert_eq!(s.len(), 3);
    ///
    /// let s = SwfStr::from_utf8_str("💖");
    /// assert_eq!(s.len(), 4);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.string.len()
    }

    /// Decodes the string into a Rust UTF-8 [`str`].
    ///
    /// The UTF-8 replacement character will be used for any invalid data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    /// use encoding_rs::UTF_8;
    ///
    /// let s = SwfStr::from_bytes(&[0xF0, 0x9F, 0x92, 0x96]);
    /// assert_eq!(s.to_str_lossy(UTF_8), "💖");
    /// ```
    #[inline]
    pub fn to_str_lossy(&self, encoding: &'static Encoding) -> Cow<'_, str> {
        encoding.decode_without_bom_handling(&self.string).0
    }

    /// Decodes the string into a Rust UTF-8 [`String`].
    ///
    /// The UTF-8 replacement character will be used for any invalid data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::SwfStr;
    /// use encoding_rs::UTF_8;
    ///
    /// let s = SwfStr::from_bytes(&[0xF0, 0x9F, 0x92, 0x96]);
    /// assert_eq!(s.to_string_lossy(UTF_8), "💖");
    /// ```
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

impl<T: ?Sized + AsRef<str>> PartialEq<T> for SwfStr {
    fn eq(&self, other: &T) -> bool {
        &self.string == other.as_ref().as_bytes()
    }
}

impl fmt::Debug for SwfStr {
    /// Formats the `SwfStr` using the given formatter.
    ///
    /// Non-ASCII characters will be formatted in hexadecimal
    /// form (`\xNN`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Write::write_char(f, '"')?;
        for chr in self
            .string
            .iter()
            .flat_map(|&c| std::ascii::escape_default(c))
        {
            fmt::Write::write_char(f, char::from(chr))?;
        }
        fmt::Write::write_char(f, '"')
    }
}
