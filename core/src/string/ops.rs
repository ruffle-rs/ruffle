use std::fmt::{self, Write};
use std::hash::Hasher;
use std::slice::Iter as SliceIter;

use super::{Units, WStr};

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

#[inline]
pub fn str_iter(s: WStr<'_>) -> Iter<'_> {
    let inner = match s.units() {
        Units::Bytes(us) => Units::Bytes(us.iter()),
        Units::Wide(us) => Units::Wide(us.iter()),
    };
    Iter { inner }
}

pub fn str_fmt(s: WStr<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    std::char::decode_utf16(s.iter())
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

pub fn str_hash<H: Hasher>(s: WStr<'_>, state: &mut H) {
    match s.units() {
        Units::Bytes(us) => us.iter().for_each(|u| state.write_u16(u16::from(*u))),
        Units::Wide(us) => us.iter().for_each(|u| state.write_u16(*u)),
    }
}
