use num_traits::Zero;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// Duration f64 nanosec
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RuffleDuration(OrderedFloat<f64>);

impl RuffleDuration {
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
        Self(OrderedFloat(nanosecs))
    }

    pub fn as_secs(&self) -> f64 {
        self.0.into_inner() / 1_000_000_000.0
    }

    pub fn as_millis(self) -> f64 {
        self.0.into_inner() / 1_000_000.0
    }

    pub fn as_micros(&self) -> f64 {
        self.0.into_inner() / 1_000.0
    }

    pub fn as_nanos(&self) -> f64 {
        self.0.into_inner()
    }

    pub fn zero() -> Self {
        Self(OrderedFloat::zero())
    }

    pub fn abs(&self) -> RuffleDuration {
        Self(OrderedFloat(self.0.abs()))
    }

    pub const fn one_sec() -> Self {
        Self(OrderedFloat(1.0))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl Add for RuffleDuration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for RuffleDuration {
    fn add_assign(&mut self, other: RuffleDuration) {
        self.0 += other.0;
    }
}

impl Sub for RuffleDuration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl SubAssign for RuffleDuration {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl From<std::time::Duration> for RuffleDuration {
    fn from(d: std::time::Duration) -> Self {
        Self::from_nanos(d.as_nanos() as f64)
    }
}

impl Into<std::time::Duration> for RuffleDuration {
    fn into(self) -> Duration {
        Duration::from_nanos(self.as_nanos() as u64)
    }
}
