use crate::{types::point::Coordinate as PointCoordinate, Point, Twips};
use std::fmt::{Display, Formatter};

pub trait Coordinate: PointCoordinate + Ord {
    const INVALID: Self;
}

impl Coordinate for Twips {
    const INVALID: Self = Self::new(0x7ffffff);
}

/// A rectangular region defined by minimum and maximum x- and y-coordinate positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rectangle<T> {
    /// The minimum x-position of the rectangle.
    pub x_min: T,

    /// The maximum x-position of the rectangle.
    pub x_max: T,

    /// The minimum y-position of the rectangle.
    pub y_min: T,

    /// The maximum y-position of the rectangle.
    pub y_max: T,
}

impl<T: PointCoordinate + Ord> Rectangle<T> {
    #[inline]
    #[must_use]
    pub fn width(&self) -> T {
        self.x_max - self.x_min
    }

    #[inline]
    pub fn set_width(&mut self, width: T) {
        self.x_max = self.x_min + width;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> T {
        self.y_max - self.y_min
    }

    #[inline]
    pub fn set_height(&mut self, height: T) {
        self.y_max = self.y_min + height;
    }

    #[must_use]
    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.x_min
            && point.x <= self.x_max
            && point.y >= self.y_min
            && point.y <= self.y_max
    }
}

impl<T: Coordinate> Rectangle<T> {
    pub const INVALID: Self = Self {
        x_min: T::INVALID,
        x_max: T::INVALID,
        y_min: T::INVALID,
        y_max: T::INVALID,
    };

    pub const ZERO: Self = Self {
        x_min: T::ZERO,
        x_max: T::ZERO,
        y_min: T::ZERO,
        y_max: T::ZERO,
    };

    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.x_min != T::INVALID
    }

    /// Clamp a given point inside this rectangle.
    #[must_use]
    pub fn clamp(&self, point: Point<T>) -> Point<T> {
        if self.is_valid() {
            Point::new(
                point.x.clamp(self.x_min, self.x_max),
                point.y.clamp(self.y_min, self.y_max),
            )
        } else {
            point
        }
    }

    #[must_use]
    pub fn encompass(mut self, point: Point<T>) -> Self {
        if self.is_valid() {
            self.x_min = self.x_min.min(point.x);
            self.x_max = self.x_max.max(point.x);
            self.y_min = self.y_min.min(point.y);
            self.y_max = self.y_max.max(point.y);
        } else {
            self.x_min = point.x;
            self.x_max = point.x;
            self.y_min = point.y;
            self.y_max = point.y;
        }
        self
    }

    #[must_use]
    pub fn union(mut self, other: &Self) -> Self {
        if !self.is_valid() {
            other.clone()
        } else {
            if other.is_valid() {
                self.x_min = self.x_min.min(other.x_min);
                self.x_max = self.x_max.max(other.x_max);
                self.y_min = self.y_min.min(other.y_min);
                self.y_max = self.y_max.max(other.y_max);
            }
            self
        }
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.is_valid()
            && self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }

    #[must_use]
    pub fn grow(mut self, amount: T) -> Self {
        if self.is_valid() {
            self.x_min -= amount;
            self.x_max += amount;
            self.y_min -= amount;
            self.y_max += amount;
        }
        self
    }

    #[must_use]
    pub fn is_point(&self) -> bool {
        self.x_min == self.x_max && self.y_min == self.y_max
    }
}

impl<T: Coordinate> Default for Rectangle<T> {
    fn default() -> Self {
        Self::INVALID
    }
}

impl<T> Display for Rectangle<T>
where
    T: Display + Coordinate,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.2}, {:.2} to {:.2}, {:.2}",
            self.x_min, self.y_min, self.x_max, self.y_max
        )
    }
}
