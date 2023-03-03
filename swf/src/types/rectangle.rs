use crate::Twips;
use std::cmp::Ord;
use std::ops::{Add, Sub};

pub trait Coordinate: Copy + Ord + Add<Output = Self> + Sub<Output = Self> {
    const INVALID: Self;
}

impl Coordinate for Twips {
    const INVALID: Self = Self::new(0x7ffffff);
}

/// A rectangular region defined by minimum and maximum x- and y-coordinate positions.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    const INVALID: Self = Self {
        x_min: T::INVALID,
        x_max: T::INVALID,
        y_min: T::INVALID,
        y_max: T::INVALID,
    };

    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.x_min != T::INVALID
    }

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

    /// Clamp a given point inside this rectangle.
    #[must_use]
    pub fn clamp(&self, (x, y): (T, T)) -> (T, T) {
        if self.is_valid() {
            (
                x.clamp(self.x_min, self.x_max),
                y.clamp(self.y_min, self.y_max),
            )
        } else {
            (x, y)
        }
    }

    #[must_use]
    pub fn encompass(mut self, x: T, y: T) -> Self {
        if self.is_valid() {
            self.x_min = self.x_min.min(x);
            self.x_max = self.x_max.max(x);
            self.y_min = self.y_min.min(y);
            self.y_max = self.y_max.max(y);
        } else {
            self.x_min = x;
            self.x_max = x;
            self.y_min = y;
            self.y_max = y;
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
    pub fn contains(&self, (x, y): (T, T)) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }
}

impl<T: Coordinate> Default for Rectangle<T> {
    fn default() -> Self {
        Self::INVALID
    }
}
