use super::pattern::Searcher;
use super::*;

use alloc::vec::Vec;
use core::fmt::Debug;

macro_rules! bstr {
    ($str:literal) => {
        WStr::from_units($str)
    };
}

macro_rules! wstr {
    ($($char:literal)*) => {
        WStr::from_units(&[$($char as u16),*])
    }
}

#[test]
fn roundtrip() {
    fn test<'a>(units: impl Into<Units<&'a [u8], &'a [u16]>>) {
        let units = units.into();
        let s = WStr::from_units(units);
        let conv = s.units();
        let eq = match (units, conv) {
            (Units::Bytes(a), Units::Bytes(b)) => a == b,
            (Units::Wide(a), Units::Wide(b)) => a == b,
            _ => false,
        };

        assert!(eq, "expected {units:?}, got {conv:?}");
    }

    test(b"");
    test(<&[u16]>::default());
    test(b"Hello!");
    test(&[
        'H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16, '!' as u16,
    ]);
}

#[test]
#[rustfmt::skip]
#[allow(clippy::eq_op)]
fn eq() {
    let a1 = bstr!(b"hello");
    let b1 = bstr!(b"world");
    let a2 = wstr!('h''e''l''l''o');
    let b2 = wstr!('w''o''r''l''d');

    assert_eq!(a1, a1); assert_eq!(a2, a1); assert_ne!(b1, a1); assert_ne!(b2, a1);
    assert_eq!(a1, a2); assert_eq!(a2, a2); assert_ne!(b1, a2); assert_ne!(b2, a2);
    assert_ne!(a1, b1); assert_ne!(a2, b1); assert_eq!(b1, b1); assert_eq!(b2, b1);
    assert_ne!(a1, b2); assert_ne!(a2, b2); assert_eq!(b1, b2); assert_eq!(b2, b2);
}

#[test]
#[rustfmt::skip]
#[allow(clippy::eq_op)]
fn cmp() {
    let a1 = bstr!(b"hello");
    let b1 = bstr!(b"world");
    let a2 = wstr!('h''e''l''l''o');
    let b2 = wstr!('w''o''r''l''d');

    assert!(a1 == a1); assert!(a2 == a1); assert!(b1 >  a1); assert!(b2 >  a1);
    assert!(a1 == a2); assert!(a2 == a2); assert!(b1 >  a2); assert!(b2 >  a2);
    assert!(a1 <  b1); assert!(a2 <  b1); assert!(b1 == b1); assert!(b2 == b1);
    assert!(a1 <  b2); assert!(a2 <  b2); assert!(b1 == b2); assert!(b2 == b2);
}

#[test]
fn fmt() {
    let a = bstr!(b"Hello world!");
    let b = wstr!('H''e''l''l''o'' ''w''o''r''l''d''!');
    let c = bstr!(b"\t\n\x03");
    let d = wstr!(0x202d 0x202e);

    assert_eq!(format!("{a}"), "Hello world!");
    assert_eq!(format!("{b}"), "Hello world!");
    assert_eq!(format!("{c}"), "\t\n\x03");
    assert_eq!(format!("{d}"), "\u{202d}\u{202e}");

    assert_eq!(format!("{a:?}"), "\"Hello world!\"");
    assert_eq!(format!("{b:?}"), "\"Hello world!\"");
    assert_eq!(format!("{c:?}"), "\"\\t\\n\\u{3}\"");
    assert_eq!(format!("{d:?}"), "\"\\u{202d}\\u{202e}\"");
}

#[test]
fn buf_concat_bytes() {
    let mut s = WString::new();
    assert_eq!(s, bstr!(b""));
    s.push_byte(b'a');
    assert_eq!(s, bstr!(b"a"));
    s.push(b'b'.into());
    assert_eq!(s, bstr!(b"ab"));
    s.push_utf8("cd");
    assert_eq!(s, bstr!(b"abcd"));
    s.push_str(bstr!(b"ef"));
    assert_eq!(s, bstr!(b"abcdef"));
    s.push_char('g');
    assert_eq!(s, bstr!(b"abcdefg"));
    assert!(matches!(s.units(), Units::Bytes(_)));
}

#[test]
fn buf_concat_wide() {
    let mut s = WString::new();
    assert_eq!(s, bstr!(b""));
    s.push_byte(b'a');
    assert_eq!(s, bstr!(b"a"));
    s.push('€' as u16);
    assert_eq!(s, wstr!('a''€'));
    s.push_utf8("😀");
    assert_eq!(s, wstr!('a''€' 0xd83d 0xde00));
    s.push_str(bstr!(b"!"));
    assert_eq!(s, wstr!('a''€' 0xd83d 0xde00 '!'));
    s.push_char('😀');
    assert_eq!(s, wstr!('a''€' 0xd83d 0xde00 '!' 0xd83d 0xde00));
    assert!(matches!(s.units(), Units::Wide(_)));
}

#[test]
fn offset_in() {
    let bstr = bstr!(b"abcdefghijk");
    assert_eq!(bstr.offset_in(bstr), Some(0));
    assert_eq!(bstr[3..6].offset_in(bstr), Some(3));
    assert_eq!(bstr.offset_in(&bstr[3..6]), None);
    assert_eq!(bstr[..3].offset_in(&bstr[6..]), None);
    assert_eq!(bstr[6..].offset_in(&bstr[..3]), None);

    let wstr = wstr!('a''b''c''d''e''f''g''h''i''j''k');
    assert_eq!(wstr.offset_in(wstr), Some(0));
    assert_eq!(wstr[3..6].offset_in(wstr), Some(3));
    assert_eq!(wstr.offset_in(&wstr[3..6]), None);
    assert_eq!(wstr[..3].offset_in(&wstr[6..]), None);
    assert_eq!(wstr[6..].offset_in(&wstr[..3]), None);

    assert_eq!(bstr.offset_in(wstr), None);
}

fn test_pattern<'a, P: Pattern<'a> + Clone + Debug>(
    haystack: &'a WStr,
    pattern: P,
    forwards: &[(usize, usize)],
    backwards: Option<&[(usize, usize)]>,
) {
    let mut searcher = pattern.clone().into_searcher(haystack);
    let mut actual: Vec<_> = core::iter::from_fn(|| searcher.next_match()).collect();
    assert_eq!(
        actual, forwards,
        "incorrect forwards matching: haystack={haystack:?}; pattern={pattern:?}",
    );

    searcher = pattern.clone().into_searcher(haystack);
    actual = core::iter::from_fn(|| searcher.next_match_back()).collect();
    actual.reverse();
    assert_eq!(
        actual,
        backwards.unwrap_or(forwards),
        "incorrect backwards matching: haystack={haystack:?}; pattern={pattern:?}",
    );
}

#[test]
fn char_patterns() {
    test_pattern(bstr!(b"a"), b'a', &[(0, 1)], None);

    let bytes = bstr!(b"abaabbcab");
    test_pattern(bytes, b'b', &[(1, 2), (4, 5), (5, 6), (8, 9)], None);
    test_pattern(bytes, b'd', &[], None);
    test_pattern(bytes, 'c' as u16, &[(6, 7)], None);
    test_pattern(bytes, '↓' as u16, &[], None);

    let wide = wstr!('↓''a''a''↓''a');
    test_pattern(wide, b'c', &[], None);
    test_pattern(wide, '↓' as u16, &[(0, 1), (3, 4)], None);
}

#[test]
fn multi_char_patterns() {
    let bytes = bstr!(b"abcdabcd");
    let matches = &[(0, 1), (2, 3), (4, 5), (6, 7)];
    test_pattern(bytes, &[b'a', b'c'][..], matches, None);
    test_pattern(bytes, &['a' as u16, 'c' as u16][..], matches, None);

    let wide = wstr!('↓''a''b''↓''b''c');
    test_pattern(wide, &[b'a', b'b'][..], &[(1, 2), (2, 3), (4, 5)], None);
    test_pattern(wide, &['↓' as u16, '−' as u16][..], &[(0, 1), (3, 4)], None);

    // Don't test `FnMut(u16) -> bool` because it isn't `Debug`
}

#[test]
fn str_patterns() {
    test_pattern(bstr!(b"aa"), bstr!(b""), &[(0, 0), (1, 1), (2, 2)], None);
    test_pattern(bstr!(b"abcde"), bstr!(b"abcde"), &[(0, 5)], None);

    let bytes = bstr!(b"bbabbbabbbba");
    let matches = &[(0, 2), (3, 5), (7, 9), (9, 11)];
    let matches_rev = &[(0, 2), (4, 6), (7, 9), (9, 11)];
    test_pattern(bytes, bstr!(b"bb"), matches, Some(matches_rev));
    test_pattern(bytes, wstr!('b''b'), matches, Some(matches_rev));

    let wide = wstr!('↓''↓''a''a''↓''↓''a''a''↓''↓');
    test_pattern(wide, bstr!(b"aa"), &[(2, 4), (6, 8)], None);
    test_pattern(wide, wstr!('↓''a'), &[(1, 3), (5, 7)], None);
}
