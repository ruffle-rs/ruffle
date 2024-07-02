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
///
/// Please be careful when using twips for calculations to avoid overflows.
/// As an example, since it takes 20 twips to get 1 pixel, 2,000 pixels are
/// 40,000 twips, or `4*10^4` . If you then have two such numbers,
/// multiplying them as part of calculations yields `16*10^8`, which is
/// relatively close to the upper limit of `i32` at about `2*10^9`.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Twips(i32);

impl Twips {
    /// There are 20 twips in a pixel.
    pub const TWIPS_PER_PIXEL: i32 = 20;

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
    pub const ONE: Self = Self(Self::TWIPS_PER_PIXEL);

    /// The `Twips` object with a value of `0.5` pixels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips::HALF.to_pixels(), 0.5);
    /// ```
    pub const HALF: Self = Self(Self::TWIPS_PER_PIXEL / 2);

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
    #[inline]
    pub const fn new(twips: i32) -> Self {
        Self(twips)
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
    #[inline]
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
    #[inline]
    pub fn from_pixels(pixels: f64) -> Self {
        Self((pixels * Self::TWIPS_PER_PIXEL as f64) as i32)
    }

    /// Converts the given number of `pixels` into twips.
    #[inline]
    pub const fn from_pixels_i32(pixels: i32) -> Self {
        Self(pixels * Self::TWIPS_PER_PIXEL)
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
    #[inline]
    pub fn to_pixels(self) -> f64 {
        f64::from(self.0) / Self::TWIPS_PER_PIXEL as f64
    }

    /// Truncates this twips to a pixel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// assert_eq!(Twips::new(23).trunc_to_pixel(), Twips::new(20));
    /// assert_eq!(Twips::new(439).trunc_to_pixel(), Twips::new(420));
    /// assert_eq!(Twips::new(-47).trunc_to_pixel(), Twips::new(-40));
    /// ```
    #[inline]
    pub fn trunc_to_pixel(self) -> Self {
        Self::new(self.0 / Twips::TWIPS_PER_PIXEL * Twips::TWIPS_PER_PIXEL)
    }

    /// Rounds this twips to the nearest pixel.
    /// Rounds half-way cases to the nearest even pixel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips;
    ///
    /// assert_eq!(Twips::new(29).round_to_pixel_ties_even(), Twips::new(20));
    /// assert_eq!(Twips::new(30).round_to_pixel_ties_even(), Twips::new(40));
    /// assert_eq!(Twips::new(31).round_to_pixel_ties_even(), Twips::new(40));
    /// ```
    #[inline]
    pub fn round_to_pixel_ties_even(self) -> Self {
        Self::from_pixels(self.to_pixels().round_ties_even())
    }
}

impl std::ops::Add for Twips {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Twips {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl std::ops::Sub for Twips {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::SubAssign for Twips {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl std::ops::Mul<i32> for Twips {
    type Output = Self;
    #[inline]
    fn mul(self, other: i32) -> Self {
        Self(self.0 * other)
    }
}

impl std::ops::MulAssign<i32> for Twips {
    #[inline]
    fn mul_assign(&mut self, other: i32) {
        self.0 *= other
    }
}

impl std::ops::Div<i32> for Twips {
    type Output = Self;
    #[inline]
    fn div(self, other: i32) -> Self {
        Self(self.0 / other)
    }
}

impl std::ops::DivAssign<i32> for Twips {
    #[inline]
    fn div_assign(&mut self, other: i32) {
        self.0 /= other
    }
}

impl std::ops::Neg for Twips {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Twips(-self.0)
    }
}

impl std::fmt::Display for Twips {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.to_pixels(), f)
    }
}
