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

/// Creates a `String` from an iterator of UTF-16 code units.
/// TODO: Unpaired surrogates will get replaced with the Unicode replacement character.
pub fn utf16_iter_to_string<I: Iterator<Item = u16>>(it: I) -> String {
    char::decode_utf16(it)
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
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

/// Maps a char to its lowercase variant according to the Flash Player.
/// Note that this mapping is different that Rust's `to_lowercase`.
pub fn swf_char_to_lowercase(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_lowercase();
    }
    let code_pt: u32 = c.into();
    if code_pt <= u16::MAX.into() {
        let code_pt = code_pt as u16;
        match LOWERCASE_TABLE.binary_search_by(|&(key, _)| key.cmp(&code_pt)) {
            Ok(i) => unsafe { char::from_u32_unchecked(LOWERCASE_TABLE[i].1.into()) },
            Err(_) => c,
        }
    } else {
        c
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

/// Maps a char to its uppercase variant according to the Flash Player.
/// Note that this mapping is different that Rust's `to_uppercase`.
pub fn swf_char_to_uppercase(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_uppercase();
    }

    let code_pt: u32 = c.into();
    if code_pt <= u16::MAX.into() {
        let code_pt = code_pt as u16;
        match UPPERCASE_TABLE.binary_search_by(|&(key, _)| key.cmp(&code_pt)) {
            Ok(i) => unsafe { char::from_u32_unchecked(UPPERCASE_TABLE[i].1.into()) },
            Err(_) => c,
        }
    } else {
        c
    }
}

pub fn swf_string_eq(a: &str, b: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        a == b
    } else {
        swf_string_eq_ignore_case(a, b)
    }
}

/// Compares two strings for equality, ignoring case as done by the Flash Player.
/// Note that the case mapping is different than Rust's case mapping.
pub fn swf_string_eq_ignore_case(a: &str, b: &str) -> bool {
    a.chars()
        .map(swf_char_to_lowercase)
        .eq(b.chars().map(swf_char_to_lowercase))
}

/// Compares two strings, ignoring case as done by the Flash Player.
/// Note that the case mapping is different than Rust's case mapping.
pub fn swf_string_cmp_ignore_case(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars()
        .map(swf_char_to_lowercase)
        .cmp(b.chars().map(swf_char_to_lowercase))
}
