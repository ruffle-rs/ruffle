pub trait Clamp {
    fn clamp_also_nan(self, min: Self, max: Self) -> Self;

    fn clamp_to_i32(self) -> i32;
}

// Extend the f64 type.
impl Clamp for f64 {
    /// A value bounded by a minimum and a maximum.
    ///
    /// `(f64::NAN).clamp(min, max)` causes the code to propagate NaN rather
    /// than returning either `max` or `min`. Instead this function returns
    /// the smallest value from the numbers provided.
    #[allow(clippy::manual_clamp)]
    fn clamp_also_nan(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    fn clamp_to_i32(self) -> i32 {
        // Clamp NaN and out-of-range (including infinite) values to `i32::MIN`.
        if self >= i32::MIN.into() && self <= i32::MAX.into() {
            self as i32
        } else {
            i32::MIN
        }
    }
}
