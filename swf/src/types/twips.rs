/// A type-safe wrapper type documenting where "twips" are used
/// in the SWF format.
///
/// A twip is 1/20th of a pixel.
/// Most coordinates in an SWF file are represented in twips.
///
/// Use the [`from_pixels`] and [`to_pixels`] methods to convert to and from
/// pixel values.
///
/// [`from_pixels`]: Twips::from_pixels
/// [`to_pixels`]: Twips::to_pixels
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Twips(i32);

impl Twips {
    /// There are 20 twips in a pixel.
    pub const TWIPS_PER_PIXEL: f64 = 20.0;

    /// The `Twips` object with a value of `0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips::ZERO.to_pixels(), 0.0);
    /// ```
    pub const ZERO: Self = Self(0);

    /// The `Twips` object with a value of `1` pixel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips::ONE.to_pixels(), 1.0);
    /// ```
    pub const ONE: Self = Self(Self::TWIPS_PER_PIXEL as i32);

    /// Creates a new `Twips` object. Note that the `twips` value is in twips,
    /// not pixels. Use the [`from_pixels`] method to convert from pixel units.
    ///
    /// [`from_pixels`]: Twips::from_pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// let twips = Twips::new(40);
    /// ```
    pub fn new<T: Into<i32>>(twips: T) -> Self {
        Self(twips.into())
    }

    /// Returns the number of twips.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// let twips = Twips::new(47);
    /// assert_eq!(twips.get(), 47);
    /// ```
    pub const fn get(self) -> i32 {
        self.0
    }

    /// Converts the given number of `pixels` into twips.
    ///
    /// This may be a lossy conversion; any precision more than a twip (1/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// // 40 pixels is equivalent to 800 twips.
    /// let twips = Twips::from_pixels(40.0);
    /// assert_eq!(twips.get(), 800);
    ///
    /// // Output is truncated if more precise than a twip (1/20 pixels).
    /// let twips = Twips::from_pixels(40.018);
    /// assert_eq!(twips.get(), 800);
    /// ```
    pub fn from_pixels(pixels: f64) -> Self {
        Self((pixels * Self::TWIPS_PER_PIXEL) as i32)
    }

    /// Converts this twips value into pixel units.
    ///
    /// This is a lossless operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// // 800 twips is equivalent to 40 pixels.
    /// let twips = Twips::new(800);
    /// assert_eq!(twips.to_pixels(), 40.0);
    ///
    /// // Twips are sub-pixel: 713 twips represent 35.65 pixels.
    /// let twips = Twips::new(713);
    /// assert_eq!(twips.to_pixels(), 35.65);
    /// ```
    pub fn to_pixels(self) -> f64 {
        f64::from(self.0) / Self::TWIPS_PER_PIXEL
    }
}

impl std::ops::Add for Twips {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Twips {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl std::ops::Sub for Twips {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::SubAssign for Twips {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl std::ops::Mul<i32> for Twips {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self(self.0 * other)
    }
}

impl std::ops::MulAssign<i32> for Twips {
    fn mul_assign(&mut self, other: i32) {
        self.0 *= other
    }
}

impl std::ops::Div<i32> for Twips {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        Self(self.0 / other)
    }
}

impl std::ops::DivAssign<i32> for Twips {
    fn div_assign(&mut self, other: i32) {
        self.0 /= other
    }
}

impl std::ops::Neg for Twips {
    type Output = Self;
    fn neg(self) -> Self {
        Twips(-self.0)
    }
}

impl std::fmt::Display for Twips {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_pixels())
    }
}
