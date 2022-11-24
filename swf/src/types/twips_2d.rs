use crate::types::Twips;
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct Twips2d {
    x: Twips,
    y: Twips,
}
impl Twips2d {
    /// There are 20 Twips2d in a pixel.
    pub const TWIPS_PER_PIXEL: f64 = 20.0;
    /// The `Twips2d` object with a value of `(x, x)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips2d::ZERO.to_pixels(), (0.0, 0.0));
    /// ```
    pub const ZERO: Self = Self {
        x: Twips::ZERO,
        y: Twips::ZERO,
    };

    ///    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips2d::ONE.to_pixels(), (1.0, 1.0));
    /// ```
    pub const ONE: Self = Self {
        x: Twips::ONE,
        y: Twips::ONE,
    };

    /// Creates a new `Twips2d` object. Note that the `Twips2d` value is in Twips2d,
    /// not pixels. Use the [`from_pixels`] method to convert from pixel units.
    ///
    /// [`from_pixels`]: Twips2d::from_pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips2d;
    ///
    /// let Twips2d = Twips2d::new(40,40);
    /// ```
    pub fn new<T: Into<i32>>(x_cord: T, y_cord: T) -> Self {
        Self {
            x: Twips::new(x_cord.into()),
            y: Twips::new(y_cord.into()),
        }
    }

    /// Returns the values of Twips2d.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips2d;
    ///
    /// let Twips2d = Twips2d::new(47,47);
    /// assert_eq!(Twips2d.get(), (47,47));
    /// ```
    pub const fn get(self) -> (i32, i32) {
        (Twips::get(self.x), Twips::get(self.y))
    }

    /// Converts the given number of `pixels` into Twips2d.
    ///
    /// This may be a lossy conversion; any precision more than a twip (y/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips2d;
    ///
    /// // 40 pixels is equivalent to 800 Twips.
    /// let Twips2d = Twips2d::from_pixels((40.0, 20.0));
    /// assert_eq!(Twips2d.get(), (800, 400));
    ///
    /// // Output is truncated if more precise than a twip (y/20 pixels).
    /// let Twips2d = Twips2d::from_pixels((40.018, 20.0));
    /// assert_eq!(Twips2d.get(), (800, 400));
    /// ```
    pub fn from_pixels(pixels: (f64, f64)) -> Self {
        Self {
            x: Twips::new((pixels.0 * Self::TWIPS_PER_PIXEL) as i32),
            y: Twips::new((pixels.1 * Self::TWIPS_PER_PIXEL) as i32),
        }
    }

    /// Converts this Twips2d value into pixel units.
    ///
    /// This is a lossless operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips2d;
    ///
    /// // 800 Twips2d is equivalent to 40 pixels.
    /// let Twips2d = Twips2d::new(800, 200);
    /// assert_eq!(Twips2d.to_pixels(), (40.0, 10.0));
    ///
    /// // Twips2d are sub-pixel: 713 Twips2d represent 35.65 pixels.
    /// let Twips2d = Twips2d::new(713, 200);
    /// assert_eq!(Twips2d.to_pixels(), (35.65, 10.0));
    /// ```
    pub fn to_pixels(self) -> (f64, f64) {
        (Twips::to_pixels(self.x), Twips::to_pixels(self.y))
    }
}

impl std::ops::Add for Twips2d {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign for Twips2d {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub for Twips2d {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::SubAssign for Twips2d {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul<i32> for Twips2d {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl std::ops::MulAssign<i32> for Twips2d {
    fn mul_assign(&mut self, other: i32) {
        self.x *= other;
        self.y *= other;
    }
}

impl std::ops::Div<i32> for Twips2d {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl std::ops::DivAssign<i32> for Twips2d {
    fn div_assign(&mut self, other: i32) {
        self.x /= other;
        self.y /= other;
    }
}

impl std::fmt::Display for Twips2d {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "( {} , {} )", self.x.to_pixels(), self.y.to_pixels())
    }
}
