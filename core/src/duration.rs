use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration as StdDuration;

/// Duration f64 nanosec
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default, Serialize, Deserialize)]
pub struct Duration(f64);

impl Duration {
    pub const ZERO: Self = Self(0.0);
    pub const ONE_SECOND: Self = Self(1.0);
    pub const MAX: Self = Self(f64::MAX);

    pub fn from_secs(secs: f64) -> Self {
        Self::from_nanos(secs * 1_000_000_000.0)
    }

    pub fn from_millis(millis: f64) -> Self {
        Self::from_nanos(millis * 1_000_000.0)
    }

    pub fn from_micros(micros: f64) -> Self {
        Self::from_nanos(micros * 1_000.0)
    }

    pub const fn from_nanos(nanosecs: f64) -> Self {
        Self(nanosecs)
    }

    pub fn as_secs(&self) -> f64 {
        self.0 / 1_000_000_000.0
    }

    pub fn as_millis(self) -> f64 {
        self.0 / 1_000_000.0
    }

    pub fn as_micros(&self) -> f64 {
        self.0 / 1_000.0
    }

    pub fn as_nanos(&self) -> f64 {
        self.0
    }

    pub fn abs(&self) -> Duration {
        Self(self.0.abs())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn min(&self, other: &Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(&self, other: &Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, other: Duration) {
        self.0 += other.0;
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl From<StdDuration> for Duration {
    fn from(d: StdDuration) -> Self {
        Self::from_nanos(d.as_nanos() as f64)
    }
}

#[allow(clippy::from_over_into)]
impl Into<StdDuration> for Duration {
    fn into(self) -> StdDuration {
        StdDuration::from_nanos(self.as_nanos() as u64)
    }
}
