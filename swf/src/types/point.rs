use crate::Twips;
use std::fmt;
use std::ops;

/// A 2D position defined by x and y coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: Twips,
    pub y: Twips,
}

impl Point {
    /// The `Point` object with a value of `(0, 0)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Point::ZERO.to_pixels(), (0.0, 0.0));
    /// ```
    pub const ZERO: Self = Self {
        x: Twips::ZERO,
        y: Twips::ZERO,
    };

    /// Creates a new `Point` object. Note that the `x` and `y` values are in twips,
    /// not pixels. Use the [`from_pixels`] method to convert from pixel units.
    ///
    /// [`from_pixels`]: Point::from_pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// let point = swf::Point::new(40, 40);
    /// ```
    pub const fn new(x: i32, y: i32) -> Self {
        Self {
            x: Twips::new(x),
            y: Twips::new(y),
        }
    }

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
    pub fn from_pixels(x: f64, y: f64) -> Self {
        Self {
            x: Twips::from_pixels(x),
            y: Twips::from_pixels(y),
        }
    }

    /// Converts this `Point` into pixel units.
    ///
    /// This is a lossless operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // 800 twips is equivalent to 40 pixels.
    /// let point = swf::Point::new(800, 200);
    /// assert_eq!(point.to_pixels(), (40.0, 10.0));
    ///
    /// // Twips are sub-pixel: 713 twips represent 35.65 pixels.
    /// let point = swf::Point::new(713, 200);
    /// assert_eq!(point.to_pixels(), (35.65, 10.0));
    /// ```
    pub fn to_pixels(self) -> (f64, f64) {
        (self.x.to_pixels(), self.y.to_pixels())
    }
}

impl ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl ops::Mul<i32> for Point {
    type Output = Self;

    fn mul(self, other: i32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::MulAssign<i32> for Point {
    fn mul_assign(&mut self, other: i32) {
        self.x *= other;
        self.y *= other;
    }
}

impl ops::Div<i32> for Point {
    type Output = Self;

    fn div(self, other: i32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl ops::DivAssign<i32> for Point {
    fn div_assign(&mut self, other: i32) {
        self.x /= other;
        self.y /= other;
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
