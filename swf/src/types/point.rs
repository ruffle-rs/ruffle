use crate::Twips;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub trait Coordinate:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<i32, Output = Self>
    + MulAssign<i32>
    + Div<i32, Output = Self>
    + DivAssign<i32>
    + Neg<Output = Self>
    + fmt::Display
{
    const ZERO: Self;
}

impl Coordinate for i32 {
    const ZERO: Self = 0;
}

impl Coordinate for Twips {
    const ZERO: Self = Self::ZERO;
}

/// A 2D position defined by x and y coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point<T: Coordinate> {
    pub x: T,
    pub y: T,
}

impl<T: Coordinate> Point<T> {
    /// The `Point` object with a value of `(0, 0)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Point::<swf::Twips>::ZERO.x, swf::Twips::ZERO);
    /// assert_eq!(swf::Point::<swf::Twips>::ZERO.y, swf::Twips::ZERO);
    /// ```
    pub const ZERO: Self = Self {
        x: T::ZERO,
        y: T::ZERO,
    };

    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<Twips> {
    /// Converts the given number of `pixels` into twips.
    ///
    /// This may be a lossy conversion; any precision more than a twip (1/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // 40 pixels is equivalent to 800 twips.
    /// let point = swf::Point::from_pixels(40.0, 20.0);
    /// assert_eq!(point.x.get(), 800);
    /// assert_eq!(point.y.get(), 400);
    ///
    /// // Output is truncated if more precise than a twip (1/20 pixels).
    /// let point = swf::Point::from_pixels(40.018, 20.0);
    /// assert_eq!(point.x.get(), 800);
    /// assert_eq!(point.y.get(), 400);
    /// ```
    #[inline]
    pub fn from_pixels(x: f64, y: f64) -> Self {
        Self {
            x: Twips::from_pixels(x),
            y: Twips::from_pixels(y),
        }
    }
}

// point + delta
impl<T: Coordinate> Add<PointDelta<T>> for Point<T> {
    type Output = Self;

    #[inline]
    fn add(self, other: PointDelta<T>) -> Self {
        Self {
            x: self.x + other.dx,
            y: self.y + other.dy,
        }
    }
}

// point += delta
impl<T: Coordinate> AddAssign<PointDelta<T>> for Point<T> {
    #[inline]
    fn add_assign(&mut self, other: PointDelta<T>) {
        self.x += other.dx;
        self.y += other.dy;
    }
}

// point - delta
impl<T: Coordinate> Sub<PointDelta<T>> for Point<T> {
    type Output = Self;

    #[inline]
    fn sub(self, other: PointDelta<T>) -> Self {
        Self {
            x: self.x - other.dx,
            y: self.y - other.dy,
        }
    }
}

// point -= delta
impl<T: Coordinate> SubAssign<PointDelta<T>> for Point<T> {
    #[inline]
    fn sub_assign(&mut self, other: PointDelta<T>) {
        self.x -= other.dx;
        self.y -= other.dy;
    }
}

// point - point
impl<T: Coordinate> Sub for Point<T> {
    type Output = PointDelta<T>;

    #[inline]
    fn sub(self, other: Self) -> PointDelta<T> {
        PointDelta {
            dx: self.x - other.x,
            dy: self.y - other.y,
        }
    }
}

impl<T: Coordinate> fmt::Display for Point<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A difference between two 2D points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointDelta<T: Coordinate> {
    pub dx: T,
    pub dy: T,
}

impl<T: Coordinate> PointDelta<T> {
    /// The `PointDelta` object with a value of `(0, 0)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::PointDelta::<swf::Twips>::ZERO.dx, swf::Twips::ZERO);
    /// assert_eq!(swf::PointDelta::<swf::Twips>::ZERO.dy, swf::Twips::ZERO);
    /// ```
    pub const ZERO: Self = Self {
        dx: T::ZERO,
        dy: T::ZERO,
    };

    pub const fn new(dx: T, dy: T) -> Self {
        Self { dx, dy }
    }
}

impl PointDelta<Twips> {
    /// Converts the given number of `pixels` into twips.
    ///
    /// This may be a lossy conversion; any precision more than a twip (1/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // 40 pixels is equivalent to 800 twips.
    /// let point = swf::PointDelta::from_pixels(40.0, 20.0);
    /// assert_eq!(point.dx.get(), 800);
    /// assert_eq!(point.dy.get(), 400);
    ///
    /// // Output is truncated if more precise than a twip (1/20 pixels).
    /// let point = swf::PointDelta::from_pixels(40.018, 20.0);
    /// assert_eq!(point.dx.get(), 800);
    /// assert_eq!(point.dy.get(), 400);
    /// ```
    #[inline]
    pub fn from_pixels(dx: f64, dy: f64) -> Self {
        Self {
            dx: Twips::from_pixels(dx),
            dy: Twips::from_pixels(dy),
        }
    }
}

// delta * i32
impl<T: Coordinate> Mul<i32> for PointDelta<T> {
    type Output = Self;

    #[inline]
    fn mul(self, other: i32) -> Self {
        Self {
            dx: self.dx * other,
            dy: self.dy * other,
        }
    }
}

// delta *= i32
impl<T: Coordinate> MulAssign<i32> for PointDelta<T> {
    #[inline]
    fn mul_assign(&mut self, other: i32) {
        self.dx *= other;
        self.dy *= other;
    }
}

// delta / i32
impl<T: Coordinate> Div<i32> for PointDelta<T> {
    type Output = Self;

    #[inline]
    fn div(self, other: i32) -> Self {
        Self {
            dx: self.dx / other,
            dy: self.dy / other,
        }
    }
}

// delta /= i32
impl<T: Coordinate> DivAssign<i32> for PointDelta<T> {
    #[inline]
    fn div_assign(&mut self, other: i32) {
        self.dx /= other;
        self.dy /= other;
    }
}

// -delta
impl<T: Coordinate> Neg for PointDelta<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            dx: -self.dx,
            dy: -self.dy,
        }
    }
}
