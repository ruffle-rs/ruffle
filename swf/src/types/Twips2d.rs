impl Twips2d {
    /// There are 20 Twips2d in a pixel.
    pub const TWIPS_PER_PIXEL: f64 = 20.0;

    /// The `Twips2d` object with a value of `(0, 0)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips2d::ZERO.to_pixels(), (0.0, 0.0));
    /// ```
    pub const ZERO: Self = Self(Twips::ZERO,Twips::ZERO);

    /// The `Twips2d` object with a value of `1` pixel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(swf::Twips2d::ONE.to_pixels(), 1.0);
    /// ```
    pub const ONE: Self = Self(Twips::ONE,Twips::ONE);

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
    pub fn new<X: Into<i32>,Y: Into<i32>>(x: X,y: Y) -> Self {
        Self( Twips::new(x.into()),Twips::new(y.into()))
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
    pub const fn get(self) -> (i32,i32) {
        (Twips::get(self.0),Twips::get(self.1))
    }

    /// Converts the given number of `pixels` into Twips2d.
    ///
    /// This may be a lossy conversion; any precision more than a twip (1/20 pixels) is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Twips2d;
    ///
    /// // 40 pixels is equivalent to 800 Twips.
    /// let Twips2d = Twips2d::from_pixels(40.0);
    /// assert_eq!(Twips2d.get(), 800);
    ///
    /// // Output is truncated if more precise than a twip (1/20 pixels).
    /// let Twips2d = Twips2d::from_pixels(40.018);
    /// assert_eq!(Twips2d.get(), 800);
    /// ```
    pub fn from_pixels(pixels:(f64,f64)) -> Self {
        Self(Twips::new((pixels.0 * Self::TWIPS_PER_PIXEL)as i32),Twips::new((pixels.1 * Self::TWIPS_PER_PIXEL)as i32))
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
    /// let Twips2d = Twips2d::new(800);
    /// assert_eq!(Twips2d.to_pixels(), 40.0);
    ///
    /// // Twips2d are sub-pixel: 713 Twips2d represent 35.65 pixels.
    /// let Twips2d = Twips2d::new(713);
    /// assert_eq!(Twips2d.to_pixels(), 35.65);
    /// ```
    pub fn to_pixels(self) -> (f64,f64) {
        (f64::from(Twips::to_pixels(self.0)),f64::from(Twips::to_pixels(self.1)) )
    }
}

impl std::ops::Add for Twips2d {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0,self.1 + other.1)
    }
}

impl std::ops::AddAssign for Twips2d {
    fn add_assign(&mut self, other: Self){
    self.0 += other.0;
    self.1 += other.1;
    }
}

impl std::ops::Sub for Twips2d {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0,self.1 - other.1)
    }
}

impl std::ops::SubAssign for Twips2d {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}

impl std::ops::Mul<i32> for Twips2d {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self(self.0 * other,self.1 * other)
    }
}

impl std::ops::MulAssign<i32> for Twips2d {
    fn mul_assign(&mut self, other: i32) {
        self.0 *= other;
        self.1 *= other;
    }
}

impl std::ops::Div<i32> for Twips2d {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        Self(self.0 / other,self.1 / other)
    }
}
impl std::ops::DivAssign<i32> for Twips2d {
    fn div_assign(&mut self, other: i32) {
        self.0 /= other;
        self.1 /= other;
    }
}

impl std::fmt::Display for Twips2d {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "( {} , {} )", self.to_pixels().0, self.to_pixels().1)
    }
}
