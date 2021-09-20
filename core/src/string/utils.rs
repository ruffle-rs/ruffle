///! Utilities for operating on strings in SWF files.
use super::tables::{LOWERCASE_TABLE, UPPERCASE_TABLE};

/// Gets the position of the previous char
/// `pos` must already lie on a char boundary
pub fn prev_char_boundary(slice: &str, pos: usize) -> usize {
    if pos == 0 {
        return pos;
    }

    let mut idx = pos - 1;
    while !slice.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Gets the byte position of the next char
/// `pos` must already lie on a char boundary
pub fn next_char_boundary(slice: &str, pos: usize) -> usize {
    if let Some(c) = slice[pos..].chars().next() {
        pos + c.len_utf8()
    } else {
        slice.len()
    }
}

/// Finds the longest prefix of `slice` that is entirely ASCII,
/// and returns it as an UTF8 string, together with the remaining tail.
pub fn split_ascii_prefix_bytes(slice: &[u8]) -> (&str, &[u8]) {
    let first_non_ascii = slice.iter().position(|c| *c >= 0x80);
    let (head, tail) = slice.split_at(first_non_ascii.unwrap_or(0));
    // SAFETY: `head` only contains ASCII.
    let head = unsafe { std::str::from_utf8_unchecked(head) };
    (head, tail)
}

/// Finds the longest prefix of `slice` that is entirely ASCII,
/// and returns it as a byte slice, together with the remaining tail.
pub fn split_ascii_prefix(slice: &str) -> (&[u8], &str) {
    let (head, tail) = split_ascii_prefix_bytes(slice.as_bytes());
    // SAFETY: `split_ascii_prefix_bytes` always split on a char boundary.
    let tail = unsafe { std::str::from_utf8_unchecked(tail) };
    (head.as_bytes(), tail)
}

/// Maps a UTF-16 code unit into a `char`.
/// TODO: Surrogate characters will get replaced with the Unicode replacement character.
pub fn utf16_code_unit_to_char(c: u16) -> char {
    char::decode_utf16(std::iter::once(c))
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
