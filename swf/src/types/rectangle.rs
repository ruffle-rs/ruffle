use std::ops::Sub;

pub trait Coordinate: Copy + Sub<Output = Self> {}

impl Coordinate for crate::Twips {}

/// A rectangular region defined by minimum and maximum x- and y-coordinate positions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rectangle<T: Coordinate> {
    /// The minimum x-position of the rectangle.
    pub x_min: T,

    /// The maximum x-position of the rectangle.
    pub x_max: T,

    /// The minimum y-position of the rectangle.
    pub y_min: T,

    /// The maximum y-position of the rectangle.
    pub y_max: T,
}

impl<T: Coordinate> Rectangle<T> {
    pub fn width(&self) -> T {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> T {
        self.y_max - self.y_min
    }
}
