use super::pattern::Searcher;
use super::*;

use alloc::vec::Vec;
use core::fmt::Debug;

macro_rules! bstr {
    ($str:expr) => {
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

#[test]
fn split_ascii_prefix() {
    assert_eq!(utils::split_ascii_prefix(""), (&b""[..], ""));
    assert_eq!(utils::split_ascii_prefix("abc"), (&b"abc"[..], ""));
    assert_eq!(utils::split_ascii_prefix("abcd€fg"), (&b"abcd"[..], "€fg"));
}

#[test]
fn char_boundary() {
    // bytes
    let bytes = bstr!(b"abcdefgh");
    assert_eq!(utils::next_char_boundary(bytes, 8), 8);
    assert_eq!(utils::prev_char_boundary(bytes, 8), 7);
    assert_eq!(utils::next_char_boundary(bytes, 7), 8);
    assert_eq!(utils::prev_char_boundary(bytes, 4), 3);
    assert_eq!(utils::next_char_boundary(bytes, 3), 4);
    assert_eq!(utils::prev_char_boundary(bytes, 1), 0);
    assert_eq!(utils::next_char_boundary(bytes, 0), 1);
    assert_eq!(utils::prev_char_boundary(bytes, 0), 0);

    // wide
    let wide = wstr!('↓''↑''a''b''c');
    assert_eq!(utils::next_char_boundary(wide, 5), 5);
    assert_eq!(utils::prev_char_boundary(wide, 5), 4);
    assert_eq!(utils::next_char_boundary(wide, 4), 5);
    assert_eq!(utils::prev_char_boundary(wide, 3), 2);
    assert_eq!(utils::next_char_boundary(wide, 2), 3);
    assert_eq!(utils::prev_char_boundary(wide, 1), 0);
    assert_eq!(utils::next_char_boundary(wide, 0), 1);
    assert_eq!(utils::prev_char_boundary(wide, 0), 0);

    // surrogate pairs
    #[rustfmt::skip]
    let sp = WStr::from_units(&[
        '↓' as u16,
        0xd83d, 0xdf01, // 🜁
        'a' as u16,
        0xd83d, 0xdf03, // 🜃
        '↓' as u16,
    ]);
    assert_eq!(utils::next_char_boundary(sp, 7), 7);
    assert_eq!(utils::prev_char_boundary(sp, 7), 6);
    assert_eq!(utils::next_char_boundary(sp, 6), 7);
    assert_eq!(utils::prev_char_boundary(sp, 6), 4);
    assert_eq!(utils::next_char_boundary(sp, 4), 6);
    assert_eq!(utils::prev_char_boundary(sp, 4), 3);
    assert_eq!(utils::next_char_boundary(sp, 3), 4);
    assert_eq!(utils::prev_char_boundary(sp, 3), 1);
    assert_eq!(utils::next_char_boundary(sp, 1), 3);
    assert_eq!(utils::prev_char_boundary(sp, 1), 0);
    assert_eq!(utils::next_char_boundary(sp, 0), 1);
    assert_eq!(utils::prev_char_boundary(sp, 0), 0);
}

#[test]
fn utf8_index_mapping() {
    #[rustfmt::skip]
    let utf16 = WStr::from_units(&[
        'a' as u16,
        'b' as u16,
        'c' as u16,
        '↓' as u16,
        'a' as u16,
        'b' as u16,
        0xd83d, 0xdf01, // 🜁
        'a' as u16,
        'ł' as u16,
        0xd83d, 0xdf03, // 🜃
        '↓' as u16,
        'a' as u16,
        'b' as u16,
        'c' as u16,
    ]);

    // utf16 indices
    // a    | b    | c    | ↓    | a    | b    | 🜁         | a    | ł    | 🜃         | ↓    | a    | b    | c
    // 0061 | 0062 | 0063 | 2193 | 0061 | 0062 | d83d df01 | 0061 | 0142 | d83d df03 | 2193 | 0061 | 0062 | 0063
    // 0    | 1    | 2    | 3    | 4    | 5    | 6    7    | 8    | 9    | 10   11   | 12   | 13   | 14   | 15

    // utf8 indices
    // a  | b  | c  | ↓        | a  | b  | 🜁           | a  | ł     | 🜃           | ↓        | a  | b  | c
    // 61 | 62 | 63 | e2 86 93 | 61 | 62 | f0 9f 9c 81 | 61 | c5 82 | f0 9f 9c 83 | e2 86 93 | 61 | 62 | 63
    // 0  | 1  | 2  | 3  4  5  | 6  | 7  | 8  9  10 11 | 12 | 13 14 | 15 16 17 18 | 19 20 21 | 22 | 23 | 24

    let to_utf8 = WStrToUtf8::new(utf16);
    let utf8 = to_utf8.to_utf8_lossy();

    assert_eq!(utf8, "abc↓ab🜁ał🜃↓abc");
    assert_eq!(utf8.len(), 25);
    assert_eq!(utf16.len(), 16);

    assert_eq!(to_utf8.utf16_index(0), Some(0));
    assert_eq!(to_utf8.utf16_index(2), Some(2));
    assert_eq!(to_utf8.utf16_index(3), Some(3));
    assert_eq!(to_utf8.utf16_index(6), Some(4));
    assert_eq!(to_utf8.utf16_index(7), Some(5));
    assert_eq!(to_utf8.utf16_index(8), Some(6));
    assert_eq!(to_utf8.utf16_index(13), Some(9));
    assert_eq!(to_utf8.utf16_index(15), Some(10));
    assert_eq!(to_utf8.utf16_index(22), Some(13));
    assert_eq!(to_utf8.utf16_index(24), Some(15));

    assert_eq!(to_utf8.utf8_index(0), Some(0));
    assert_eq!(to_utf8.utf8_index(2), Some(2));
    assert_eq!(to_utf8.utf8_index(3), Some(3));
    assert_eq!(to_utf8.utf8_index(4), Some(6));
    assert_eq!(to_utf8.utf8_index(5), Some(7));
    assert_eq!(to_utf8.utf8_index(6), Some(8));
    assert_eq!(to_utf8.utf8_index(9), Some(13));
    assert_eq!(to_utf8.utf8_index(10), Some(15));
    assert_eq!(to_utf8.utf8_index(13), Some(22));
    assert_eq!(to_utf8.utf8_index(15), Some(24));

    // last (potential) position
    assert_eq!(to_utf8.utf16_index(25), Some(16));
    assert_eq!(to_utf8.utf8_index(16), Some(25));

    // out of bounds
    assert_eq!(to_utf8.utf16_index(26), None);
    assert_eq!(to_utf8.utf8_index(17), None);

    // indices outside of character boundary
    assert_eq!(to_utf8.utf16_index(4), Some(4));
    assert_eq!(to_utf8.utf16_index(5), Some(4));
    assert_eq!(to_utf8.utf16_index(9), Some(8));
    assert_eq!(to_utf8.utf16_index(10), Some(8));
    assert_eq!(to_utf8.utf8_index(7), Some(12));
    assert_eq!(to_utf8.utf8_index(11), Some(19));
}

#[test]
fn utf8_index_mapping_empty() {
    let utf16 = WStr::empty();

    let to_utf8 = WStrToUtf8::new(utf16);
    let utf8 = to_utf8.to_utf8_lossy();

    assert_eq!(utf8.len(), 0);
    assert_eq!(utf16.len(), 0);

    assert_eq!(to_utf8.utf16_index(0), Some(0));
    assert_eq!(to_utf8.utf16_index(1), None);
}

#[test]
fn parse() {
    fn test_u32(string: &[u8]) {
        let actual = bstr!(string).parse::<u32>();
        if let Ok(expected) = core::str::from_utf8(string).unwrap().parse::<u32>() {
            assert!(actual.is_ok());
            assert_eq!(actual.unwrap(), expected);
        } else {
            assert!(actual.is_err());
        }
    }

    fn test_i32(string: &[u8]) {
        let actual = bstr!(string).parse::<i32>();
        if let Ok(expected) = core::str::from_utf8(string).unwrap().parse::<i32>() {
            assert!(actual.is_ok());
            assert_eq!(actual.unwrap(), expected);
        } else {
            assert!(actual.is_err());
        }
    }

    test_u32(b"0");
    test_u32(b"123");
    test_u32(b"001");
    test_u32(b"123asd");
    test_u32(b"asdf");
    test_u32(b"");
    test_u32(b"   ");
    test_u32(b"123");
    test_u32(b"4294967295");
    test_u32(b"4294967296");
    test_u32(b"4294967297");
    test_u32(b"+0");
    test_u32(b"+1");
    test_u32(b"-1");
    test_u32(b"-0");
    test_u32(b"-");
    test_u32(b"+");

    test_i32(b"0");
    test_i32(b"123");
    test_i32(b"001");
    test_i32(b"123asd");
    test_i32(b"asdf");
    test_i32(b"");
    test_i32(b"   ");
    test_i32(b"123");
    test_i32(b"4294967295");
    test_i32(b"4294967296");
    test_i32(b"4294967297");
    test_i32(b"+0");
    test_i32(b"+1");
    test_i32(b"-1");
    test_i32(b"-0");
    test_i32(b"-");
    test_i32(b"+");
}
