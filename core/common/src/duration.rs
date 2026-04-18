//! A custom Duration type for representing time intervals with f64 precision.
//!
//! This type is used instead of `std::time::Duration` because some calculations
//! require higher precision than what `std::time::Duration` (nanosecond resolution)
//! provides. For example, audio calculations may need ~16 significant figures
//! of precision, which f64 provides.

use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A duration of time represented in milliseconds with f64 precision.
///
/// This type provides type safety for time intervals while maintaining
/// the precision needed for calculations that require f64 precision.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct FloatDuration(f64);

impl FloatDuration {
    /// A duration of zero.
    pub const ZERO: Self = Self(0.0);

    /// Creates a new `FloatDuration` from milliseconds.
    #[inline]
    #[must_use]
    pub const fn from_millis(millis: f64) -> Self {
        Self(millis)
    }

    /// Creates a new `FloatDuration` from seconds.
    #[inline]
    #[must_use]
    pub fn from_secs(secs: f64) -> Self {
        Self(secs * 1000.0)
    }

    /// Creates a `FloatDuration` from a `std::time::Duration`.
    #[inline]
    #[must_use]
    pub fn from_std(duration: std::time::Duration) -> Self {
        Self(duration.as_nanos() as f64 / 1_000_000.0)
    }

    /// Returns the duration in milliseconds.
    #[inline]
    #[must_use]
    pub const fn as_millis(&self) -> f64 {
        self.0
    }

    /// Converts to a `std::time::Duration`.
    ///
    /// Note: This may lose precision for very small durations.
    #[inline]
    #[must_use]
    pub fn to_std(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64((self.0 / 1000.0).max(0.0))
    }

    /// Returns the minimum of two durations.
    #[inline]
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        if self.0 <= other.0 { self } else { other }
    }

    /// Returns the maximum of two durations.
    #[inline]
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self.0 >= other.0 { self } else { other }
    }
}

impl Add for FloatDuration {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for FloatDuration {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for FloatDuration {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for FloatDuration {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_millis() {
        let d = FloatDuration::from_millis(1500.0);
        assert_eq!(d.as_millis(), 1500.0);
    }

    #[test]
    fn test_from_secs() {
        let d = FloatDuration::from_secs(1.5);
        assert_eq!(d.as_millis(), 1500.0);
    }

    #[test]
    fn test_from_std() {
        let std_duration = std::time::Duration::from_millis(1500);
        let d = FloatDuration::from_std(std_duration);
        assert_eq!(d.as_millis(), 1500.0);
    }

    #[test]
    fn test_to_std() {
        let d = FloatDuration::from_millis(1500.0);
        assert_eq!(d.to_std(), std::time::Duration::from_millis(1500));
    }

    #[test]
    fn test_arithmetic() {
        let a = FloatDuration::from_millis(100.0);
        let b = FloatDuration::from_millis(50.0);

        assert_eq!((a + b).as_millis(), 150.0);
        assert_eq!((a - b).as_millis(), 50.0);
    }

    #[test]
    fn test_min_max() {
        let a = FloatDuration::from_millis(100.0);
        let b = FloatDuration::from_millis(50.0);

        assert_eq!(a.min(b).as_millis(), 50.0);
        assert_eq!(a.max(b).as_millis(), 100.0);
    }
}
