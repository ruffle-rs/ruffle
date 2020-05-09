//! Tests for HTML module

use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use swf::{Rectangle, Twips};

#[test]
fn position_add() {
    let pos1 = Position::from((12, 31));
    let pos2 = Position::from((1, -160));

    assert_eq!(pos1 + pos2, Position::from((13, -129)));
}

#[test]
fn position_add_assign() {
    let mut pos1 = Position::from((12, 31));
    let pos2 = Position::from((1, -160));

    pos1 += pos2;

    assert_eq!(pos1, Position::from((13, -129)));
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(20), Twips::new(80)))
        )
    )
}

#[test]
fn bounds_extents() {
    let pos = Position::from((Twips::new(3), Twips::new(5)));
    let size = Size::from((Twips::new(20), Twips::new(80)));
    let bounds = BoxBounds::from_position_and_size(pos, size);
    let extent_pos = Position::from((bounds.extent_x(), bounds.extent_y()));

    assert_eq!(extent_pos, Position::from((Twips::new(23), Twips::new(85))));
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(57), Twips::new(80)))
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(57), Twips::new(80)))
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(57), Twips::new(80)))
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(57), Twips::new(80)))
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
            Position::from((Twips::new(1), Twips::new(25))),
            Size::from((Twips::new(20), Twips::new(80)))
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
            Position::from((Twips::new(1), Twips::new(25))),
            Size::from((Twips::new(20), Twips::new(80)))
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(18), Twips::new(100)))
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
            Position::from((Twips::new(3), Twips::new(5))),
            Size::from((Twips::new(18), Twips::new(100)))
        )
    );
}

#[test]
fn textformat_merge() {
    let mut tf1 = TextFormat::default();
    tf1.font = Some("First".to_string());
    tf1.size = Some(10.0);
    tf1.bold = None;
    tf1.italic = None;

    let mut tf2 = TextFormat::default();
    tf2.font = Some("Second".to_string());
    tf2.size = Some(10.0);
    tf2.bold = Some(false);
    tf2.italic = None;

    let merged = tf1.merge_matching_properties(tf2);

    assert_eq!(merged.font, None);
    assert_eq!(merged.size, Some(10.0));
    assert_eq!(merged.bold, None);
    assert_eq!(merged.italic, None);
}

#[test]
fn formatspans_resolve_position() {
    let fs = FormatSpans::from_str_and_spans(
        "abcdefghi",
        &[
            TextSpan::with_length(2),
            TextSpan::with_length(3),
            TextSpan::with_length(1),
            TextSpan::with_length(1),
            TextSpan::with_length(2),
        ],
    );

    assert_eq!(Some(0), fs.resolve_position_as_span(0));
    assert_eq!(Some(0), fs.resolve_position_as_span(1));
    assert_eq!(Some(1), fs.resolve_position_as_span(2));
    assert_eq!(Some(1), fs.resolve_position_as_span(3));
    assert_eq!(Some(1), fs.resolve_position_as_span(4));
    assert_eq!(Some(2), fs.resolve_position_as_span(5));
    assert_eq!(Some(3), fs.resolve_position_as_span(6));
    assert_eq!(Some(4), fs.resolve_position_as_span(7));
    assert_eq!(Some(4), fs.resolve_position_as_span(8));
    assert_eq!(None, fs.resolve_position_as_span(9));
}

#[test]
fn formatspans_get_text_format() {
    let mut tf1 = TextFormat::default();
    tf1.font = Some("Same!".to_string());
    tf1.size = Some(12.0);

    let mut tf2 = TextFormat::default();
    tf2.font = Some("Difference!".to_string());
    tf2.size = Some(12.0);

    let fs = FormatSpans::from_str_and_spans(
        "abcdefghi",
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
