use alloc::borrow::{Borrow, Cow};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Write};
use core::hash::Hasher;
use core::slice::Iter as SliceIter;

use super::pattern::{SearchStep, Searcher};
use super::{utils, Pattern, Units, WStr, WString};

pub struct Iter<'a> {
    inner: Units<SliceIter<'a, u8>, SliceIter<'a, u16>>,
}

impl Iterator for Iter<'_> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Units::Bytes(it) => it.next().map(|c| *c as u16),
            Units::Wide(it) => it.next().copied(),
        }
    }
}

impl DoubleEndedIterator for Iter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Units::Bytes(it) => it.next_back().map(|c| *c as u16),
            Units::Wide(it) => it.next_back().copied(),
        }
    }
}

pub type Chars<'a> = core::char::DecodeUtf16<Iter<'a>>;

pub struct CharIndices<'a> {
    chars: Chars<'a>,
    start: usize,
}

impl Iterator for CharIndices<'_> {
    type Item = (usize, Result<char, core::char::DecodeUtf16Error>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let res = (self.start, self.chars.next()?);

        let in_bmp = match &res.1 {
            Ok(c) => u32::from(*c) <= u16::MAX.into(),
            Err(_) => false,
        };

        self.start += if in_bmp { 1 } else { 2 };
        Some(res)
    }
}

#[inline]
pub fn str_iter(s: &WStr) -> Iter<'_> {
    let inner = match s.units() {
        Units::Bytes(us) => Units::Bytes(us.iter()),
        Units::Wide(us) => Units::Wide(us.iter()),
    };
    Iter { inner }
}

#[inline]
pub fn str_char_indices(s: &WStr) -> CharIndices<'_> {
    CharIndices {
        chars: s.chars(),
        start: 0,
    }
}

pub fn str_fmt(s: &WStr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let utf8 = WStrToUtf8::new(s);
    f.write_str(utf8.head)?;
    utf8.tail
        .chars()
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .try_for_each(|c| f.write_char(c))
}

pub fn str_debug_fmt(s: &WStr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;

    for c in core::char::decode_utf16(s.iter()) {
        match c {
            Ok(c) => c.escape_debug().try_for_each(|c| f.write_char(c))?,
            Err(err) => write!(f, "\\u{{{:x}}}", err.unpaired_surrogate())?,
        }
    }

    f.write_char('"')
}

pub fn str_eq(left: &WStr, right: &WStr) -> bool {
    if core::ptr::eq(left, right) {
        return true;
    }

    let (bytes, wide) = match (left.units(), right.units()) {
        (Units::Bytes(a), Units::Bytes(b)) => return a == b,
        (Units::Wide(a), Units::Wide(b)) => return a == b,
        (Units::Bytes(a), Units::Wide(b)) => (a, b),
        (Units::Wide(a), Units::Bytes(b)) => (b, a),
    };

    if bytes.len() != wide.len() {
        return false;
    }

    (0..bytes.len()).all(|i| {
        // SAFETY: Both slices have the same length.
        unsafe { *bytes.get_unchecked(i) as u16 == *wide.get_unchecked(i) }
    })
}

pub fn str_eq_ignore_case(left: &WStr, right: &WStr) -> bool {
    let left = left.iter().map(utils::swf_to_lowercase);
    let right = right.iter().map(utils::swf_to_lowercase);
    left.eq(right)
}

pub fn str_cmp(left: &WStr, right: &WStr) -> core::cmp::Ordering {
    let (bytes, wide, rev) = match (left.units(), right.units()) {
        (Units::Bytes(a), Units::Bytes(b)) => return a.cmp(b),
        (Units::Wide(a), Units::Wide(b)) => return a.cmp(b),
        (Units::Bytes(a), Units::Wide(b)) => (a, b, false),
        (Units::Wide(a), Units::Bytes(b)) => (b, a, true),
    };

    let bytes = bytes.iter().map(|c| *c as u16);
    let wide = wide.iter().copied();
    let cmp = bytes.cmp(wide);
    if rev {
        cmp.reverse()
    } else {
        cmp
    }
}

pub fn str_cmp_ignore_case(left: &WStr, right: &WStr) -> core::cmp::Ordering {
    let left = left.iter().map(utils::swf_to_lowercase);
    let right = right.iter().map(utils::swf_to_lowercase);
    left.cmp(right)
}

pub fn str_hash<H: Hasher>(s: &WStr, state: &mut H) {
    state.write_u32(s.len() as u32);
    match s.units() {
        // Using `state.write_bytes(us)` would be incorrect here, as `Hash`
        // doesn't guarantee any equivalence between its various methods.
        Units::Bytes(us) => us.iter().for_each(|u| state.write_u8(*u)),
        Units::Wide(us) => us.iter().for_each(|u| {
            if *u <= 0xFF {
                state.write_u8(*u as u8)
            } else {
                state.write_u16(*u)
            }
        }),
    }
}

pub fn str_offset_in(s: &WStr, other: &WStr) -> Option<usize> {
    let offset = match (s.units(), other.units()) {
        (Units::Bytes(a), Units::Bytes(b)) => {
            (a.as_ptr() as usize).checked_sub(b.as_ptr() as usize)
        }
        (Units::Wide(a), Units::Wide(b)) => (a.as_ptr() as usize)
            .checked_sub(b.as_ptr() as usize)
            .map(|n| n / core::mem::size_of::<u16>()),
        _ => None,
    };

    offset.filter(|o| o + s.len() <= other.len())
}

fn map_latin1_chars(s: &WStr, mut map: impl FnMut(u8) -> u8) -> WString {
    match s.units() {
        Units::Bytes(us) => {
            let us: Vec<u8> = us.iter().map(|c| map(*c)).collect();
            WString::from_buf(us)
        }
        Units::Wide(us) => {
            let us: Vec<u16> = us
                .iter()
                .map(|c| match u8::try_from(*c) {
                    Ok(c) => map(c).into(),
                    Err(_) => *c,
                })
                .collect();
            WString::from_buf(us)
        }
    }
}

pub fn str_to_ascii_lowercase(s: &WStr) -> WString {
    map_latin1_chars(s, |c| c.to_ascii_lowercase())
}

pub fn str_make_ascii_lowercase(s: &mut WStr) {
    match s.units_mut() {
        Units::Bytes(us) => us.make_ascii_lowercase(),
        Units::Wide(us) => {
            for c in us {
                if let Ok(b) = u8::try_from(*c) {
                    *c = b.to_ascii_lowercase().into();
                }
            }
        }
    }
}

pub fn str_to_ascii_uppercase(s: &WStr) -> WString {
    map_latin1_chars(s, |c| c.to_ascii_uppercase())
}

pub fn str_make_ascii_uppercase(s: &mut WStr) {
    match s.units_mut() {
        Units::Bytes(us) => us.make_ascii_uppercase(),
        Units::Wide(us) => {
            for c in us {
                if let Ok(b) = u8::try_from(*c) {
                    *c = b.to_ascii_uppercase().into();
                }
            }
        }
    }
}

pub fn str_is_latin1(s: &WStr) -> bool {
    match s.units() {
        Units::Bytes(_) => true,
        Units::Wide(us) => us.iter().all(|c| *c <= u16::from(u8::MAX)),
    }
}

pub fn str_join<E: Borrow<WStr>>(elems: &[E], sep: &WStr) -> WString {
    fn join_inner<T, E, F>(total_len: usize, elems: &[E], sep: &WStr, mut extend: F) -> Vec<T>
    where
        E: Borrow<WStr>,
        F: FnMut(&mut Vec<T>, &WStr),
    {
        let mut buf = Vec::with_capacity(total_len);
        extend(&mut buf, elems[0].borrow());
        for e in &elems[1..] {
            extend(&mut buf, sep);
            extend(&mut buf, e.borrow());
        }
        buf
    }

    if elems.is_empty() {
        return WString::default();
    }

    let (len, is_latin1) = elems.iter().fold(
        (sep.len() * elems.len().saturating_sub(1), sep.is_latin1()),
        |(len, is_latin1), e| {
            let e = e.borrow();
            (len + e.len(), is_latin1 && e.is_latin1())
        },
    );

    if is_latin1 {
        let buf = join_inner(len, elems, sep, |buf: &mut Vec<u8>, e| match e.units() {
            Units::Bytes(us) => buf.extend_from_slice(us),
            Units::Wide(us) => buf.extend(us.iter().map(|c| *c as u8)),
        });
        WString::from_buf(buf)
    } else {
        let buf = join_inner(len, elems, sep, |buf: &mut Vec<u16>, e| match e.units() {
            Units::Bytes(us) => buf.extend(us.iter().map(|c| *c as u16)),
            Units::Wide(us) => buf.extend_from_slice(us),
        });
        WString::from_buf(buf)
    }
}

pub fn str_repeat(s: &WStr, count: usize) -> WString {
    if count == 0 || s.is_empty() {
        return WString::new();
    }

    let len = s.len().saturating_mul(count);
    if len > WStr::MAX_LEN {
        super::panic_on_invalid_length(len);
    }

    match (s.units(), s.is_latin1()) {
        (Units::Bytes(us), _) => WString::from_buf(us.repeat(count)),
        (Units::Wide(us), false) => WString::from_buf(us.repeat(count)),
        (Units::Wide(us), true) => {
            let mut buf = Vec::with_capacity(len);
            buf.extend(us.iter().map(|c| *c as u8));
            while buf.len() <= len / 2 {
                buf.extend_from_within(..);
            }
            buf.extend_from_within(..(len - buf.len()));
            WString::from_buf(buf)
        }
    }
}

pub fn str_replace<'a, P: Pattern<'a>>(haystack: &'a WStr, pattern: P, with: &WStr) -> WString {
    let mut result = WString::new();
    let mut prev_end = 0;

    let mut searcher = pattern.into_searcher(haystack);
    while let Some((start, end)) = searcher.next_match() {
        result.push_str(&haystack[prev_end..start]);
        result.push_str(with);
        prev_end = end;
    }
    result.push_str(&haystack[prev_end..]);

    result
}

pub fn str_find<'a, P: Pattern<'a>>(haystack: &'a WStr, pattern: P) -> Option<usize> {
    pattern
        .into_searcher(haystack)
        .next_match()
        .map(|(start, _)| start)
}

pub fn str_rfind<'a, P: Pattern<'a>>(haystack: &'a WStr, pattern: P) -> Option<usize> {
    pattern
        .into_searcher(haystack)
        .next_match_back()
        .map(|(start, _)| start)
}

#[inline]
pub fn str_split<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> Split<'a, P> {
    Split {
        string: Some(string),
        searcher: pattern.into_searcher(string),
        prev_end: 0,
    }
}

pub fn str_split_once<'a, P: Pattern<'a>>(
    string: &'a WStr,
    pattern: P,
) -> Option<(&'a WStr, &'a WStr)> {
    let (start, end) = pattern.into_searcher(string).next_match()?;
    Some((&string[..start], &string[end..]))
}

pub fn str_rsplit_once<'a, P: Pattern<'a>>(
    string: &'a WStr,
    pattern: P,
) -> Option<(&'a WStr, &'a WStr)> {
    let (start, end) = pattern.into_searcher(string).next_match_back()?;
    Some((&string[..start], &string[end..]))
}

pub fn starts_with<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> bool {
    matches!(
        pattern.into_searcher(string).next(),
        SearchStep::Match(_, _)
    )
}

pub fn ends_with<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> bool {
    matches!(
        pattern.into_searcher(string).next_back(),
        SearchStep::Match(_, _)
    )
}

pub fn strip_prefix<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> Option<&'a WStr> {
    match pattern.into_searcher(string).next() {
        SearchStep::Match(_, end) => Some(&string[end..]),
        _ => None,
    }
}

pub fn strip_suffix<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> Option<&'a WStr> {
    match pattern.into_searcher(string).next_back() {
        SearchStep::Match(start, _) => Some(&string[..start]),
        _ => None,
    }
}

pub fn str_trim_matches<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> &'a WStr {
    let mut i = 0;
    let mut j = 0;
    let mut searcher = pattern.into_searcher(string);
    if let Some((start, end)) = searcher.next_reject() {
        i = start;
        j = end;
    }

    if let Some((_, end)) = searcher.next_reject_back() {
        j = end;
    }

    &string[i..j]
}

pub fn str_trim_start_matches<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> &'a WStr {
    let mut i = string.len();
    let mut searcher = pattern.into_searcher(string);
    if let Some((start, _)) = searcher.next_reject() {
        i = start;
    }

    &string[i..]
}

pub fn str_trim_end_matches<'a, P: Pattern<'a>>(string: &'a WStr, pattern: P) -> &'a WStr {
    let mut i = 0;
    let mut searcher = pattern.into_searcher(string);
    if let Some((_, end)) = searcher.next_reject_back() {
        i = end;
    }

    &string[..i]
}

pub struct Split<'a, P: Pattern<'a>> {
    string: Option<&'a WStr>,
    searcher: P::Searcher,
    prev_end: usize,
}

impl<'a, P: Pattern<'a>> Iterator for Split<'a, P> {
    type Item = &'a WStr;

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.string?;

        match self.searcher.next_match() {
            Some((start, end)) => {
                let end = core::mem::replace(&mut self.prev_end, end);
                Some(&string[end..start])
            }
            None => {
                self.string = None;
                Some(&string[self.prev_end..])
            }
        }
    }
}

/// A struct for converting a `WStr` to an UTF8 `String`.
pub struct WStrToUtf8<'a> {
    head: &'a str,
    tail: &'a WStr,
}

impl<'a> WStrToUtf8<'a> {
    pub fn new(s: &'a WStr) -> Self {
        let (head, tail) = match s.units() {
            Units::Bytes(b) => {
                let (head, tail) = utils::split_ascii_prefix_bytes(b);
                (head, WStr::from_units(tail))
            }
            Units::Wide(_) => ("", s),
        };

        Self { head, tail }
    }

    pub fn to_utf8_lossy(&self) -> Cow<'a, str> {
        if self.tail.is_empty() {
            Cow::Borrowed(self.head)
        } else {
            let mut out = String::with_capacity(self.head.len() + self.tail.len());
            out.push_str(self.head);
            write!(out, "{}", self.tail).unwrap();
            Cow::Owned(out)
        }
    }

    /// Map the given UTF-16 code unit index to its corresponding UTF-8 code unit index.
    pub fn utf8_index(&self, utf16_index: usize) -> Option<usize> {
        self.translate_index(utf16_index, false)
            .map(|(utf8_index, _)| utf8_index)
    }

    /// Map the given UTF-8 code unit index to its corresponding UTF-16 code unit index.
    pub fn utf16_index(&self, utf8_index: usize) -> Option<usize> {
        self.translate_index(utf8_index, true)
            .map(|(_, utf16_index)| utf16_index)
    }

    fn translate_index(&self, index: usize, is_utf8: bool) -> Option<(usize, usize)> {
        let ascii_prefix_len = self.head.len();
        if index <= ascii_prefix_len {
            return Some((index, index));
        }

        if self.tail.is_empty() {
            return None;
        }

        let mut utf8_tail_pos = 0;
        let mut utf16_tail_pos = 0;

        while if is_utf8 {
            utf8_tail_pos + ascii_prefix_len < index
        } else {
            utf16_tail_pos + ascii_prefix_len < index
        } {
            let c = self.tail[utf16_tail_pos..].chars().next()?.ok()?;
            utf8_tail_pos += c.len_utf8();
            utf16_tail_pos += c.len_utf16();
        }

        Some((
            ascii_prefix_len + utf8_tail_pos,
            ascii_prefix_len + utf16_tail_pos,
        ))
    }

    #[inline]
    pub fn prefix(&self) -> &str {
        self.head
    }
}
