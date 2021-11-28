//! Tests for HTML module

use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::string::{WStr, WString};
use swf::{Rectangle, Twips};

#[test]
fn position_add() {
    let pos1 = Position::from((12, 31));
    let pos2 = Position::from((1, -160));

    assert_eq!(pos1 + pos2, (13, -129).into());
}

#[test]
fn position_add_assign() {
    let mut pos1 = Position::from((12, 31));
    let pos2 = Position::from((1, -160));

    pos1 += pos2;

    assert_eq!(pos1, (13, -129).into());
}

#[test]
fn into_swf_rectangle() {
    let pos = Position::from((Twips::new(3), Twips::new(5)));
    let size = Size::from((Twips::new(20), Twips::new(80)));
    let bounds = BoxBounds::from_position_and_size(pos, size);
    let rect: Rectangle = bounds.into();

    assert_eq!(
        rect,
        Rectangle {
            x_min: Twips::new(3),
            x_max: Twips::new(23),
            y_min: Twips::new(5),
            y_max: Twips::new(85),
        }
    );
}

#[test]
fn from_swf_rectangle() {
    let rect = Rectangle {
        x_min: Twips::new(3),
        x_max: Twips::new(23),
        y_min: Twips::new(5),
        y_max: Twips::new(85),
    };
    let bounds = BoxBounds::from(rect);
    let (pos, size) = bounds.into_position_and_size();

    assert_eq!(
        (pos, size),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(20), Twips::new(80)).into(),
        )
    )
}

#[test]
fn bounds_extents() {
    let pos = Position::from((Twips::new(3), Twips::new(5)));
    let size = Size::from((Twips::new(20), Twips::new(80)));
    let bounds = BoxBounds::from_position_and_size(pos, size);
    let extent_pos = Position::from((bounds.extent_x(), bounds.extent_y()));

    assert_eq!(extent_pos, (Twips::new(23), Twips::new(85)).into());
}

#[test]
fn bounds_union() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(50), Twips::new(75)));
    let size2 = Size::from((Twips::new(10), Twips::new(3)));
    let bounds2 = BoxBounds::from_position_and_size(pos2, size2);

    let union = bounds1 + bounds2;

    assert_eq!(
        union.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(57), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_unionassign() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let mut bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(50), Twips::new(75)));
    let size2 = Size::from((Twips::new(10), Twips::new(3)));
    let bounds2 = BoxBounds::from_position_and_size(pos2, size2);

    bounds1 += bounds2;

    assert_eq!(
        bounds1.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(57), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_union_reverse() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(50), Twips::new(75)));
    let size2 = Size::from((Twips::new(10), Twips::new(3)));
    let bounds2 = BoxBounds::from_position_and_size(pos2, size2);

    let union = bounds2 + bounds1;

    assert_eq!(
        union.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(57), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_unionassign_reverse() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(50), Twips::new(75)));
    let size2 = Size::from((Twips::new(10), Twips::new(3)));
    let mut bounds2 = BoxBounds::from_position_and_size(pos2, size2);

    bounds2 += bounds1;

    assert_eq!(
        bounds2.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(57), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_position_add() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(-2), Twips::new(20)));

    let bounds2 = bounds1 + pos2;

    assert_eq!(
        bounds2.into_position_and_size(),
        (
            (Twips::new(1), Twips::new(25)).into(),
            (Twips::new(20), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_position_addassign() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let mut bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let pos2 = Position::from((Twips::new(-2), Twips::new(20)));

    bounds1 += pos2;

    assert_eq!(
        bounds1.into_position_and_size(),
        (
            (Twips::new(1), Twips::new(25)).into(),
            (Twips::new(20), Twips::new(80)).into(),
        )
    );
}

#[test]
fn bounds_size_add() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let size2 = Size::from((Twips::new(-2), Twips::new(20)));

    let bounds2 = bounds1 + size2;

    assert_eq!(
        bounds2.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(18), Twips::new(100)).into(),
        )
    );
}

#[test]
fn bounds_size_addassign() {
    let pos1 = Position::from((Twips::new(3), Twips::new(5)));
    let size1 = Size::from((Twips::new(20), Twips::new(80)));
    let mut bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let size2 = Size::from((Twips::new(-2), Twips::new(20)));

    bounds1 += size2;

    assert_eq!(
        bounds1.into_position_and_size(),
        (
            (Twips::new(3), Twips::new(5)).into(),
            (Twips::new(18), Twips::new(100)).into(),
        )
    );
}

#[test]
fn bounds_with_size() {
    let pos1 = Position::from((Twips::ZERO, Twips::new(5760)));
    let size1 = Size::from((Twips::new(7900), Twips::new(500)));
    let bounds1 = BoxBounds::from_position_and_size(pos1, size1);

    let size2 = Size::from((Twips::new(7780), Twips::new(500)));
    let bounds2 = bounds1.with_size(size2);

    assert_eq!(bounds2.into_position_and_size(), (pos1, size2))
}

#[test]
fn textformat_merge() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("First")),
        size: Some(10.0),
        bold: None,
        italic: None,
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("Second")),
        size: Some(10.0),
        bold: Some(false),
        italic: None,
        ..Default::default()
    };

    let merged = tf1.merge_matching_properties(tf2);

    assert_eq!(merged.font, None);
    assert_eq!(merged.size, Some(10.0));
    assert_eq!(merged.bold, None);
    assert_eq!(merged.italic, None);
}

#[test]
fn textformat_mix() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("first")),
        size: Some(10.0),
        bold: None,
        italic: None,
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("second")),
        size: Some(10.0),
        bold: Some(false),
        italic: None,
        ..Default::default()
    };

    let mixed = tf1.mix_with(tf2);

    assert_eq!(mixed.font, Some(WString::from_utf8("first")));
    assert_eq!(mixed.size, Some(10.0));
    assert_eq!(mixed.bold, Some(false));
    assert_eq!(mixed.italic, None);
}

#[test]
fn formatspans_set_default() {
    let mut fs = FormatSpans::new();

    let tf1 = TextFormat {
        font: Some(WString::from_utf8("first")),
        size: Some(10.0),
        bold: None,
        italic: None,
        ..Default::default()
    };

    fs.set_default_format(tf1);

    let out_tf1 = fs.default_format();

    assert_eq!(out_tf1.font, Some(WString::from_utf8("first")));
    assert_eq!(out_tf1.size, Some(10.0));
    assert_eq!(out_tf1.bold, None);
    assert_eq!(out_tf1.italic, None);

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("second")),
        size: Some(10.0),
        bold: Some(false),
        italic: None,
        ..Default::default()
    };

    fs.set_default_format(tf2);

    let out_tf2 = fs.default_format();

    assert_eq!(out_tf2.font, Some(WString::from_utf8("second")));
    assert_eq!(out_tf2.size, Some(10.0));
    assert_eq!(out_tf2.bold, Some(false));
    assert_eq!(out_tf2.italic, None);
}

#[test]
fn formatspans_resolve_position() {
    let fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(2, Default::default()),
            TextSpan::with_length_and_format(3, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(2, Default::default()),
        ],
    );

    assert_eq!(Some((0, 0)), fs.resolve_position_as_span(0));
    assert_eq!(Some((0, 1)), fs.resolve_position_as_span(1));
    assert_eq!(Some((1, 0)), fs.resolve_position_as_span(2));
    assert_eq!(Some((1, 1)), fs.resolve_position_as_span(3));
    assert_eq!(Some((1, 2)), fs.resolve_position_as_span(4));
    assert_eq!(Some((2, 0)), fs.resolve_position_as_span(5));
    assert_eq!(Some((3, 0)), fs.resolve_position_as_span(6));
    assert_eq!(Some((4, 0)), fs.resolve_position_as_span(7));
    assert_eq!(Some((4, 1)), fs.resolve_position_as_span(8));
    assert_eq!(None, fs.resolve_position_as_span(9));
}

#[test]
fn formatspans_ensure_span_break() {
    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(2, Default::default()),
            TextSpan::with_length_and_format(3, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(2, Default::default()),
        ],
    );

    let breakpoint = fs.ensure_span_break_at(4);
    assert_eq!(Some(2), breakpoint);

    assert_eq!(Some((0, 0)), fs.resolve_position_as_span(0));
    assert_eq!(Some((0, 1)), fs.resolve_position_as_span(1));
    assert_eq!(Some((1, 0)), fs.resolve_position_as_span(2));
    assert_eq!(Some((1, 1)), fs.resolve_position_as_span(3));
    assert_eq!(Some((2, 0)), fs.resolve_position_as_span(4));
    assert_eq!(Some((3, 0)), fs.resolve_position_as_span(5));
    assert_eq!(Some((4, 0)), fs.resolve_position_as_span(6));
    assert_eq!(Some((5, 0)), fs.resolve_position_as_span(7));
    assert_eq!(Some((5, 1)), fs.resolve_position_as_span(8));
    assert_eq!(None, fs.resolve_position_as_span(9));
}

#[test]
fn formatspans_ensure_span_break_redundant() {
    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(2, Default::default()),
            TextSpan::with_length_and_format(3, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(2, Default::default()),
        ],
    );

    let breakpoint = fs.ensure_span_break_at(5);
    assert_eq!(Some(2), breakpoint);

    assert_eq!(Some((0, 0)), fs.resolve_position_as_span(0));
    assert_eq!(Some((0, 1)), fs.resolve_position_as_span(1));
    assert_eq!(Some((1, 0)), fs.resolve_position_as_span(2));
    assert_eq!(Some((1, 1)), fs.resolve_position_as_span(3));
    assert_eq!(Some((1, 2)), fs.resolve_position_as_span(4));
    assert_eq!(Some((2, 0)), fs.resolve_position_as_span(5));
    assert_eq!(Some((3, 0)), fs.resolve_position_as_span(6));
    assert_eq!(Some((4, 0)), fs.resolve_position_as_span(7));
    assert_eq!(Some((4, 1)), fs.resolve_position_as_span(8));
    assert_eq!(None, fs.resolve_position_as_span(9));
}

#[test]
fn formatspans_span_boundaries() {
    let fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(2, Default::default()),
            TextSpan::with_length_and_format(3, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(1, Default::default()),
            TextSpan::with_length_and_format(2, Default::default()),
        ],
    );

    assert_eq!((0, 5), fs.get_span_boundaries(0, 9));
    assert_eq!((0, 3), fs.get_span_boundaries(1, 6));
    assert_eq!((1, 2), fs.get_span_boundaries(2, 3));
    assert_eq!((1, 5), fs.get_span_boundaries(3, 11));
    assert_eq!((1, 4), fs.get_span_boundaries(4, 7));
    assert_eq!((2, 5), fs.get_span_boundaries(5, 8));
}

#[test]
fn formatspans_get_text_format() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        size: Some(12.0),
        ..Default::default()
    };

    let fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(2, tf1.clone()),
            TextSpan::with_length_and_format(3, tf1.clone()),
            TextSpan::with_length_and_format(1, tf2),
            TextSpan::with_length_and_format(1, tf1.clone()),
            TextSpan::with_length_and_format(2, tf1.clone()),
        ],
    );

    let first_span = fs.get_text_format(0, 1);
    assert!(first_span.font.is_some());
    assert_eq!(tf1.size, first_span.size);

    let before_the_break = fs.get_text_format(0, 5);
    assert!(before_the_break.font.is_some());
    assert_eq!(tf1.size, before_the_break.size);

    let all = fs.get_text_format(0, 100);
    assert!(all.font.is_none());
    assert_eq!(tf1.size, all.size);
}

#[test]
fn formatspans_normalize_no_spans() {
    let mut fs = FormatSpans::from_str_and_spans(WStr::from_units(b"abcdefghi"), &[]);

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 9));
}

#[test]
fn formatspans_normalize_no_text() {
    let mut fs = FormatSpans::from_str_and_spans(WStr::empty(), &[]);

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 0));
    assert_eq!(None, fs.resolve_position_as_span(5));
}

#[test]
fn formatspans_normalize_short_spans() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(1, tf2),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 6));
    assert_eq!((2, 3), fs.get_span_boundaries(6, 9));
}

#[test]
fn formatspans_normalize_exact_spans() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 9));
    assert_eq!(None, fs.resolve_position_as_span(50));
}

#[test]
fn formatspans_normalize_long_spans() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1.clone()),
            TextSpan::with_length_and_format(2000, tf2),
            TextSpan::with_length_and_format(5, tf1),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 9));
    assert_eq!(None, fs.resolve_position_as_span(50));
}

#[test]
fn formatspans_normalize_merge_spans() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1.clone()),
            TextSpan::with_length_and_format(4, tf1),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 9));
}

#[test]
fn formatspans_normalize_merge_many_spans() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(1, tf1.clone()),
            TextSpan::with_length_and_format(1, tf1.clone()),
            TextSpan::with_length_and_format(2, tf1.clone()),
            TextSpan::with_length_and_format(1, tf1.clone()),
            TextSpan::with_length_and_format(1, tf1.clone()),
            TextSpan::with_length_and_format(3, tf1),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 9));
}

#[test]
fn formatspans_normalize_long_spans_with_merge() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        size: Some(12.0),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        size: Some(12.0),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1.clone()),
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(2000, tf2),
        ],
    );

    fs.normalize();

    assert_eq!((0, 1), fs.get_span_boundaries(0, 9));
    assert_eq!(None, fs.resolve_position_as_span(50));
}

#[test]
fn formatspans_normalize_set_text_format_double_cut() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    let tf3 = TextFormat {
        font: Some(WString::from_utf8("Another difference!")),
        ..Default::default()
    };

    fs.set_text_format(3, 7, &tf3);

    assert_eq!((0, 1), fs.get_span_boundaries(0, 3));
    assert_eq!((1, 2), fs.get_span_boundaries(3, 7));
    assert_eq!((2, 3), fs.get_span_boundaries(7, 9));
}

#[test]
fn formatspans_normalize_set_text_format_single_cut() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    let tf3 = TextFormat {
        font: Some(WString::from_utf8("Another difference!")),
        ..Default::default()
    };

    fs.set_text_format(3, 5, &tf3);

    assert_eq!((0, 1), fs.get_span_boundaries(0, 3));
    assert_eq!((1, 2), fs.get_span_boundaries(3, 5));
    assert_eq!((2, 3), fs.get_span_boundaries(5, 9));
}

#[test]
fn formatspans_normalize_set_text_format_no_cut() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    let tf3 = TextFormat {
        font: Some(WString::from_utf8("Another difference!")),
        ..Default::default()
    };

    fs.set_text_format(0, 5, &tf3);

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 9));
}

#[test]
fn formatspans_replace_text_inbounds() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    fs.replace_text(3, 6, WStr::from_units(b"123"), None);

    assert_eq!(WStr::from_units(b"abc123ghi"), fs.text());

    assert_eq!((0, 1), fs.get_span_boundaries(0, 3));
    assert_eq!((1, 2), fs.get_span_boundaries(3, 9));
}

#[test]
fn formatspans_replace_text_edgebounds() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    fs.replace_text(8, 35, WStr::from_units(b"123"), None);

    assert_eq!(WStr::from_units(b"abcdefgh123"), fs.text());

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 8));
    assert_eq!((2, 3), fs.get_span_boundaries(8, 11));
}

#[test]
fn formatspans_replace_text_oob() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    fs.replace_text(24, 35, WStr::from_units(b"123"), None);

    assert_eq!(WStr::from_units(b"abcdefghi123"), fs.text());

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 9));
    assert_eq!((2, 3), fs.get_span_boundaries(9, 12));
}

#[test]
fn formatspans_replace_text_degenerate() {
    let tf1 = TextFormat {
        font: Some(WString::from_utf8("same!")),
        ..Default::default()
    };

    let tf2 = TextFormat {
        font: Some(WString::from_utf8("difference!")),
        ..Default::default()
    };

    let mut fs = FormatSpans::from_str_and_spans(
        WStr::from_units(b"abcdefghi"),
        &[
            TextSpan::with_length_and_format(5, tf1),
            TextSpan::with_length_and_format(4, tf2),
        ],
    );

    fs.replace_text(52, 35, WStr::from_units(b"123"), None);

    assert_eq!(WStr::from_units(b"abcdefghi"), fs.text());

    assert_eq!((0, 1), fs.get_span_boundaries(0, 5));
    assert_eq!((1, 2), fs.get_span_boundaries(5, 9));
}
