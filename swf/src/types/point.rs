#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct points(f32, f32);
impl Point {

    /// The `Point` object with a value of `(0, 0)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Point::ZERO.to_pixels(), (0.0, 0.0));
    /// ```
    pub const ZERO: Self = Self(0.0,0.0);

    /// The `Point` object with a value of `1` pixel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Point::ONE.to_pixels(), 1.0);
    /// ```
    pub const ONE: Self = Self(1.0,1.0);

    /// Creates a new `Point` object. Note that the `Point` value is in Point,
    /// not pixels. Use the [`from_pixels`] method to convert from pixel units.
    ///
    /// [`from_pixels`]: Point::from_pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Point;
    ///
    /// let Point = Point::new(40,40);
    /// ```
    pub fn new<X:,Y>(x: X,y: Y) -> Self {
        Self(x,y)
    }

    /// Returns the values of Point.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Point;
    ///
    /// let Point = Point::new(47,47);
    /// assert_eq!(Point.get(), (47,47));
    /// ```
    pub const fn get(self) -> (i32,i32) {
        (Twips::get(self.0),Twips::get(self.1))
    }

    /// Converts the given number of `pixels` into Point.
    ///
    /// This may be a lossy conversion; any precision more than a twip (1/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Point;
    ///
    /// // 40 pixels is equivalent to 800 Twips.
    /// let Point = Point::from_pixels(40.0);
    /// assert_eq!(Point.get(), 800);
    ///
    /// // Output is truncated if more precise than a twip (1/20 pixels).
    /// let Point = Point::from_pixels(40.018);
    /// assert_eq!(Point.get(), 800);
    /// ```
    pub fn from_pixels(pixels:(f64,f64)) -> Self {
        Self(Twips::new((pixels.0 * Self::TWIPS_PER_PIXEL)as i32),Twips::new((pixels.1 * Self::TWIPS_PER_PIXEL)as i32))
    }

    /// Converts this Point value into pixel units.
    ///
    /// This is a lossless operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Point;
    ///
    /// // 800 Point is equivalent to 40 pixels.
    /// let Point = Point::new(800);
    /// assert_eq!(Point.to_pixels(), 40.0);
    ///
    /// // Point are sub-pixel: 713 Point represent 35.65 pixels.
    /// let Point = Point::new(713);
    /// assert_eq!(Point.to_pixels(), 35.65);
    /// ```
    pub fn to_pixels(self) -> (f64,f64) {
        (f64::from(Twips::to_pixels(self.0)),f64::from(Twips::to_pixels(self.1)) )
    }

    /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric bounds
    /// of [`i32`] instead of overflowing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Point;
    ///
    /// assert_eq!(Point::new(40).saturating_sub(Point::new(20)), Point::new(20));
    /// assert_eq!(Point::new(i32::MIN).saturating_sub(Point::new(5)), Point::new(i32::MIN));
    /// assert_eq!(Point::new(i32::MAX).saturating_sub(Point::new(-100)), Point::new(i32::MAX));
    /// ```
    #[must_use]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0),self.1.saturating_sub(rhs.0))
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0,self.1 + other.1)
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, other: Self){
    self.0 += other.0;
    self.1 += other.1;
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0,self.1 - other.1)
    }
}

impl std::ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}

impl std::ops::Mul<i32> for Point {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self(self.0 * other,self.1 * other)
    }
}

impl std::ops::MulAssign<i32> for Point {
    fn mul_assign(&mut self, other: i32) {
        self.0 *= other;
        self.1 *= other;
    }
}

impl std::ops::Div<i32> for Point {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        Self(self.0 / other,self.1 / other)
    }
}
impl std::ops::DivAssign<i32> for Point {
    fn div_assign(&mut self, other: i32) {
        self.0 /= other;
        self.1 /= other;
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "( {} , {} )", self.to_pixels().0, self.to_pixels().1)
    }
}
