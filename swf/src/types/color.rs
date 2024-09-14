/// An RGBA (red, green, blue, alpha) color.
///
/// All components are stored as [`u8`] and have a color range of 0-255.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Color {
    /// The red component value.
    pub r: u8,

    /// The green component value.
    pub g: u8,

    /// The blue component value.
    pub b: u8,

    /// The alpha component value.
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::from_rgb(0, 0);
    pub const BLACK: Self = Self::from_rgb(0, 255);
    pub const GRAY: Self = Self::from_rgb(0x555555, 255);
    pub const WHITE: Self = Self::from_rgb(0xFFFFFF, 255);
    pub const RED: Self = Self::from_rgb(0xFF0000, 255);
    pub const GREEN: Self = Self::from_rgb(0x00FF00, 255);
    pub const BLUE: Self = Self::from_rgb(0x0000FF, 255);
    pub const YELLOW: Self = Self::from_rgb(0xFFFF00, 255);
    pub const CYAN: Self = Self::from_rgb(0x00FFFF, 255);
    pub const MAGENTA: Self = Self::from_rgb(0xFF00FF, 255);

    /// Creates a `Color` from a 32-bit `rgb` value and an `alpha` value.
    ///
    /// The byte-ordering of the 32-bit `rgb` value is XXRRGGBB.
    /// The most significant byte, represented by XX, is ignored;
    /// the `alpha` value is provided separately.
    /// This is followed by the red (RR), green (GG), and blue (BB) components values,
    /// respectively.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Color;
    ///
    /// let red = Color::from_rgb(0xFF0000, 255);
    /// let green = Color::from_rgb(0x00FF00, 255);
    /// let blue = Color::from_rgb(0x0000FF, 255);
    /// ```
    #[inline]
    pub const fn from_rgb(rgb: u32, alpha: u8) -> Self {
        let [b, g, r, _] = rgb.to_le_bytes();
        Self { r, g, b, a: alpha }
    }

    /// Creates a `Color` from a 32-bit `rgba` value.
    ///
    /// The byte-ordering of the 32-bit `rgba` value is AARRGGBB.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swf::Color;
    ///
    /// let red = Color::from_rgba(0xFFFF0000);
    /// let green = Color::from_rgba(0xFF00FF00);
    /// let blue = Color::from_rgba(0xFF0000FF);
    /// ```
    #[inline]
    pub const fn from_rgba(rgba: u32) -> Self {
        let [b, g, r, a] = rgba.to_le_bytes();
        Self { r, g, b, a }
    }

    /// Converts the color to a 32-bit RGB value.
    ///
    /// The alpha value does not get stored.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```rust
    /// use swf::Color;
    ///
    /// let color = Color::from_rgb(0xFF00FF, 255);
    /// assert_eq!(color.to_rgb(), 0xFF00FF);
    /// ```
    ///
    /// Alpha values do not get stored:
    /// ```rust
    /// use swf::Color;
    ///
    /// let color1 = Color::from_rgb(0xFF00FF, 255);
    /// let color2 = Color::from_rgb(0xFF00FF, 0);
    /// assert_eq!(color1.to_rgb(), color2.to_rgb());
    /// ```
    #[inline]
    pub const fn to_rgb(&self) -> u32 {
        u32::from_le_bytes([self.b, self.g, self.r, 0])
    }

    /// Converts the color to a 32-bit RGBA value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```rust
    /// use swf::Color;
    ///
    /// let color = Color::from_rgb(0xFF00FF, 255);
    /// assert_eq!(color.to_rgba(), 0xFFFF00FF);
    /// ```
    #[inline]
    pub const fn to_rgba(&self) -> u32 {
        u32::from_le_bytes([self.b, self.g, self.r, self.a])
    }
}
