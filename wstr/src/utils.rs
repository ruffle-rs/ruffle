//! Utilities for operating on strings in SWF files.

use super::tables::{LOWERCASE_TABLE, UPPERCASE_TABLE};
use super::Units;
use alloc::vec::Vec;

fn is_surrogate_pair_at(us: &[u16], pos: usize) -> bool {
    if let Some(pair) = us.get(pos..pos + 2) {
        let has_high = (0xD800..=0xDBFF).contains(&pair[0]);
        let has_low = (0xDC00..=0xDFFF).contains(&pair[1]);
        has_high && has_low
    } else {
        false
    }
}

/// Gets the position of the previous utf16 char;
/// `pos` must already lie on a char boundary
pub fn prev_char_boundary(slice: &super::WStr, pos: usize) -> usize {
    if pos <= 1 {
        return 0;
    }

    match slice.units() {
        Units::Bytes(_) => pos - 1, // LATIN1 strings only contains 1-bytes chars
        Units::Wide(us) if is_surrogate_pair_at(us, pos - 2) => pos - 2,
        Units::Wide(_) => pos - 1,
    }
}

/// Gets the byte position of the next utf16 char;
/// `pos` must already lie on a char boundary
pub fn next_char_boundary(slice: &super::WStr, pos: usize) -> usize {
    if pos >= slice.len() {
        return slice.len();
    }

    match slice.units() {
        Units::Bytes(_) => pos + 1, // LATIN1 strings only contains 1-bytes chars
        Units::Wide(us) if is_surrogate_pair_at(us, pos) => pos + 2,
        Units::Wide(_) => pos + 1,
    }
}

/// Returns `true` if the given utf16 code unit is a whitespace
/// according to the Flash Player.
#[inline]
pub fn swf_is_whitespace(c: u16) -> bool {
    matches!(u8::try_from(c), Ok(b' ' | b'\t' | b'\n' | b'\r'))
}

/// Returns `true` if the given utf16 code unit is a newline
/// according to the Flash Player.
#[inline]
pub fn swf_is_newline(c: u16) -> bool {
    matches!(u8::try_from(c), Ok(b'\n' | b'\r'))
}

/// Finds the longest prefix of `slice` that is entirely ASCII,
/// and returns it as an UTF8 string, together with the remaining tail.
pub fn split_ascii_prefix_bytes(slice: &[u8]) -> (&str, &[u8]) {
    let first_non_ascii = slice.iter().position(|c| *c >= 0x80);
    let (head, tail) = slice.split_at(first_non_ascii.unwrap_or(slice.len()));
    // SAFETY: `head` only contains ASCII.
    let head = unsafe { core::str::from_utf8_unchecked(head) };
    (head, tail)
}

/// Finds the longest prefix of `slice` that is entirely ASCII,
/// and returns it as a byte slice, together with the remaining tail.
pub fn split_ascii_prefix(slice: &str) -> (&[u8], &str) {
    let (head, tail) = split_ascii_prefix_bytes(slice.as_bytes());
    // SAFETY: `split_ascii_prefix_bytes` always split on a char boundary.
    let tail = unsafe { core::str::from_utf8_unchecked(tail) };
    (head.as_bytes(), tail)
}

/// Maps a UTF-16 code unit into a `char`.
/// TODO: Surrogate characters will get replaced with the Unicode replacement character.
pub fn utf16_code_unit_to_char(c: u16) -> char {
    char::decode_utf16(core::iter::once(c))
        .next()
        .unwrap()
        .unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// Maps a UCS2 code unit to its lowercase variant according to the Flash Player.
/// Note that this mapping is different that Rust's `to_lowercase`.
pub fn swf_to_lowercase(c: u16) -> u16 {
    if c < 0x80 {
        return (c as u8).to_ascii_lowercase().into();
    }

    match LOWERCASE_TABLE.binary_search_by(|&(key, _)| key.cmp(&c)) {
        Ok(i) => LOWERCASE_TABLE[i].1,
        Err(_) => c,
    }
}

/// Maps a UCS2 code unit to its uppercase variant according to the Flash Player.
/// Note that this mapping is different that Rust's `to_uppercase`.
pub fn swf_to_uppercase(c: u16) -> u16 {
    if c < 0x80 {
        return (c as u8).to_ascii_uppercase().into();
    }

    match UPPERCASE_TABLE.binary_search_by(|&(key, _)| key.cmp(&c)) {
        Ok(i) => UPPERCASE_TABLE[i].1,
        Err(_) => c,
    }
}

/// This is the same idea as std::str::Chars, except it uses flash's weird UTF-8 decoding rules,
/// and works on raw bytes. It also does not return `char`, but raw u32's that may or may not be valid chars.
///
/// The main difference between UTF-8 decoding in flash and regular UTF-8 decoding is that invalid UTF-8 sequences
/// are interpreted as if they were LATIN1 characters. Flash also completely ignores the rule about UTF-8 sequences
/// not being allowed to be in surrogate range (0xD800-0xDBFF, 0xDC00-0xDFFF).
///
/// Another difference is that if a multibyte sequence is expecting 4 bytes, rather than failing/replacing with a
/// replacement character since the maximum is 3, Flash will instead just only read the next 3 bytes and completely
/// ignore the fact that the starting byte was expecting 4.
pub struct DecodeAvmUtf8<'a> {
    src: &'a [u8],
    index: usize,
}

impl<'a> DecodeAvmUtf8<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self { src, index: 0 }
    }
}

impl Iterator for DecodeAvmUtf8<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let first = *self.src.get(self.index)?;
        let ones = first.leading_ones();
        self.index += 1;

        if ones <= 1 {
            return Some(first as u32);
        }

        let mb_count = core::cmp::min(ones - 1, 3);
        let bm = u8::MAX >> ones;
        let mut ch = (bm & first) as u32;
        match self
            .src
            .get(self.index..)
            .and_then(|src| src.get(..mb_count as usize))
        {
            Some(mb) => {
                for b in mb.iter() {
                    // continuation bytes should start with a single leading 1
                    if b.leading_ones() != 1 {
                        return Some(first as u32);
                    }
                    ch <<= 6;
                    ch |= (*b & (u8::MAX >> 2)) as u32;
                }
                if ch < 0x80 {
                    Some(first as u32)
                } else {
                    self.index += mb_count as usize;
                    debug_assert!(ch <= 0x10FFFF);
                    Some(ch)
                }
            }
            None => Some(first as u32),
        }
    }
}

/// Encodes a raw character point into UTF16. Unlike char::encode_utf16, this does not require
/// that the character point is valid.
pub fn encode_raw_utf16(mut ch: u32, dst: &mut Vec<u16>) {
    if ch < 0x10000 {
        dst.push(ch as u16);
        return;
    }
    ch -= 0x10000;
    let mut w1: u16 = 0xD800;
    let mut w2: u16 = 0xDC00;
    w1 |= (ch >> 10) as u16;
    w2 |= (ch & !(u32::MAX << 10)) as u16;
    dst.extend_from_slice(&[w1, w2]);
}
