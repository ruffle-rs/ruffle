use std::borrow::Cow;
use std::fmt::{self, Write};
use std::hash::Hasher;
use std::slice::Iter as SliceIter;

use super::pattern::{SearchStep, Searcher};
use super::{utils, BorrowWStr, Pattern, Units, WStr, WString};

pub struct Iter<'a> {
    inner: Units<SliceIter<'a, u8>, SliceIter<'a, u16>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Units::Bytes(it) => it.next().map(|c| *c as u16),
            Units::Wide(it) => it.next().copied(),
        }
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Units::Bytes(it) => it.next_back().map(|c| *c as u16),
            Units::Wide(it) => it.next_back().copied(),
        }
    }
}

#[inline]
pub fn str_iter(s: WStr<'_>) -> Iter<'_> {
    let inner = match s.units() {
        Units::Bytes(us) => Units::Bytes(us.iter()),
        Units::Wide(us) => Units::Wide(us.iter()),
    };
    Iter { inner }
}

pub fn str_fmt(s: WStr<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let utf8 = WStrToUtf8::new(s);
    f.write_str(utf8.head)?;
    std::char::decode_utf16(utf8.tail.iter())
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .try_for_each(|c| f.write_char(c))
}

pub fn str_debug_fmt(s: WStr<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;

    for c in std::char::decode_utf16(s.iter()) {
        match c {
            Ok(c) => c.escape_debug().try_for_each(|c| f.write_char(c))?,
            Err(err) => write!(f, "\\u{{{:x}}}", err.unpaired_surrogate())?,
        }
    }

    f.write_char('"')
}

pub fn str_eq(left: WStr<'_>, right: WStr<'_>) -> bool {
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

pub fn str_eq_ignore_case(left: WStr<'_>, right: WStr<'_>) -> bool {
    let left = left.iter().map(utils::swf_to_lowercase);
    let right = right.iter().map(utils::swf_to_lowercase);
    left.eq(right)
}

pub fn str_cmp(left: WStr<'_>, right: WStr<'_>) -> std::cmp::Ordering {
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

pub fn str_cmp_ignore_case(left: WStr<'_>, right: WStr<'_>) -> std::cmp::Ordering {
    let left = left.iter().map(utils::swf_to_lowercase);
    let right = right.iter().map(utils::swf_to_lowercase);
    left.cmp(right)
}

pub fn str_hash<H: Hasher>(s: WStr<'_>, state: &mut H) {
    match s.units() {
        Units::Bytes(us) => us.iter().for_each(|u| state.write_u16(u16::from(*u))),
        Units::Wide(us) => us.iter().for_each(|u| state.write_u16(*u)),
    }
}

fn map_latin1_chars(s: WStr<'_>, mut map: impl FnMut(u8) -> u8) -> WString {
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

pub fn str_to_ascii_lowercase(s: WStr<'_>) -> WString {
    map_latin1_chars(s, |c| c.to_ascii_lowercase())
}

pub fn str_is_latin1(s: WStr<'_>) -> bool {
    match s.units() {
        Units::Bytes(_) => true,
        Units::Wide(us) => us.iter().all(|c| *c <= u16::from(u8::MAX)),
    }
}

pub fn str_join<E: BorrowWStr>(elems: &[E], sep: WStr<'_>) -> WString {
    fn join_inner<T, E, F>(total_len: usize, elems: &[E], sep: WStr<'_>, mut extend: F) -> Vec<T>
    where
        E: BorrowWStr,
        F: FnMut(&mut Vec<T>, WStr<'_>),
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

pub fn str_find<'a, P: Pattern<'a>>(haystack: WStr<'a>, pattern: P) -> Option<usize> {
    pattern
        .into_searcher(haystack)
        .next_match()
        .map(|(start, _)| start)
}

pub fn str_rfind<'a, P: Pattern<'a>>(haystack: WStr<'a>, pattern: P) -> Option<usize> {
    pattern
        .into_searcher(haystack)
        .next_match_back()
        .map(|(start, _)| start)
}

#[inline]
pub fn str_split<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> Split<'a, P> {
    Split {
        string: Some(string),
        searcher: pattern.into_searcher(string),
        prev_end: 0,
    }
}

pub fn str_rsplit_once<'a, P: Pattern<'a>>(
    string: WStr<'a>,
    pattern: P,
) -> Option<(WStr<'a>, WStr<'a>)> {
    let (start, end) = pattern.into_searcher(string).next_match_back()?;
    Some((string.slice(..start), string.slice(end..)))
}

pub fn starts_with<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> bool {
    matches!(
        pattern.into_searcher(string).next(),
        SearchStep::Match(_, _)
    )
}

pub fn ends_with<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> bool {
    matches!(
        pattern.into_searcher(string).next_back(),
        SearchStep::Match(_, _)
    )
}

pub fn strip_prefix<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> Option<WStr<'a>> {
    match pattern.into_searcher(string).next() {
        SearchStep::Match(_, end) => Some(string.slice(end..)),
        _ => None,
    }
}

pub fn strip_suffix<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> Option<WStr<'a>> {
    match pattern.into_searcher(string).next_back() {
        SearchStep::Match(start, _) => Some(string.slice(..start)),
        _ => None,
    }
}

pub fn str_trim_matches<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> WStr<'a> {
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

    string.slice(i..j)
}

pub fn str_trim_start_matches<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> WStr<'a> {
    let mut i = string.len();
    let mut searcher = pattern.into_searcher(string);
    if let Some((start, _)) = searcher.next_reject() {
        i = start;
    }

    string.slice(i..)
}

pub fn str_trim_end_matches<'a, P: Pattern<'a>>(string: WStr<'a>, pattern: P) -> WStr<'a> {
    let mut i = 0;
    let mut searcher = pattern.into_searcher(string);
    if let Some((_, end)) = searcher.next_reject_back() {
        i = end;
    }

    string.slice(..i)
}

pub struct Split<'a, P: Pattern<'a>> {
    string: Option<WStr<'a>>,
    searcher: P::Searcher,
    prev_end: usize,
}

impl<'a, P: Pattern<'a>> Iterator for Split<'a, P> {
    type Item = WStr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.string?;

        match self.searcher.next_match() {
            Some((start, end)) => {
                let end = std::mem::replace(&mut self.prev_end, end);
                Some(string.slice(end..start))
            }
            None => {
                self.string = None;
                Some(string.slice(self.prev_end..))
            }
        }
    }
}

/// A struct for converting a `WStr<'_>` to an UTF8 `String`.
pub struct WStrToUtf8<'a> {
    head: &'a str,
    tail: WStr<'a>,
}

impl<'a> WStrToUtf8<'a> {
    pub fn new(s: WStr<'a>) -> Self {
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

    #[inline]
    pub fn prefix(&self) -> &str {
        self.head
    }
}
