//! Provides UCS2 string types for usage in AVM1 and AVM2.
//!
//! Internally, these types are represented by a sequence of 1-byte or 2-bytes (wide) code units,
//! that may contains null bytes or unpaired surrogates.
//!
//! To match Flash behavior, the string length is limited to 2³¹-1 code units;
//! any attempt to create a longer string will panic.

#![no_std]

#[cfg_attr(test, macro_use)]
extern crate alloc;

#[macro_use]
mod common;

mod buf;
mod ops;
mod parse;
mod pattern;
pub mod ptr;
mod tables;
pub mod utils;

#[cfg(test)]
mod tests;

pub use buf::WString;
pub use common::{Units, WStr};
pub use ops::{CharIndices, Chars, Iter, Split, WStrToUtf8};
pub use parse::{FromWStr, Integer};
pub use pattern::Pattern;
pub use ptr::WStrMetadata;

use alloc::borrow::Cow;
use core::borrow::Borrow;

pub use common::panic_on_invalid_length;

/// Flattens a slice of strings, placing `sep` as a separator between each.
#[inline]
pub fn join<E: Borrow<WStr>, S: Borrow<WStr>>(elems: &[E], sep: &S) -> WString {
    crate::ops::str_join(elems, sep.borrow())
}

/// Converts a borrowed UTF-8 string to a `WStr` slice.
#[inline]
pub fn from_utf8(s: &str) -> Cow<'_, WStr> {
    let (ascii, tail) = utils::split_ascii_prefix(s);
    if tail.is_empty() {
        // We can directly reinterpret ASCII bytes as LATIN1.
        Cow::Borrowed(WStr::from_units(ascii))
    } else {
        Cow::Owned(WString::from_utf8_inner(ascii, tail))
    }
}

/// Converts a slice of UTF-8 bytes to a `WStr` slice.
///
/// Invalid UTF-8 sequences are treated as described in `utils::DecodeAvmUtf8`.
pub fn from_utf8_bytes(bytes: &[u8]) -> Cow<'_, WStr> {
    let (ascii, tail) = utils::split_ascii_prefix_bytes(bytes);
    if tail.is_empty() {
        // We can directly reinterpret ASCII bytes as LATIN1.
        Cow::Borrowed(WStr::from_units(bytes))
    } else {
        Cow::Owned(WString::from_utf8_bytes_inner(ascii, tail))
    }
}
