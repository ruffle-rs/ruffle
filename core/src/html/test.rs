//! Tests for HTML module

use crate::html::dimensions::{BoxBounds, Position, Size};
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
