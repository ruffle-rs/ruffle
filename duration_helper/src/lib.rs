use std::ops::{Add, Neg, Sub};
use std::time::Duration;

pub enum Sign {
    Positive,
    Negative,
}

pub struct SignedDuration {
    sign: Sign,
    duration: Duration,
}

impl SignedDuration {
    pub fn new(sign: Sign, duration: Duration) -> Self {
        SignedDuration { sign, duration }
    }

    pub fn from_secs(secs: i64) -> Self {
        SignedDuration::new(
            if secs < 0 { Sign::Negative } else { Sign::Positive },
            Duration::from_secs(secs.abs() as u64),
        )
    }

    pub fn from_f64_secs(secs: f64) -> Self {
        SignedDuration::new(
            if secs < 0.0 { Sign::Negative } else { Sign::Positive },
            Duration::from_nanos((secs.abs() * 1_000_000_000.0) as u64),
        )
    }

    pub fn from_nanos(nanosecs: i64) -> Self {
        SignedDuration::new(
            if nanosecs < 0 { Sign::Negative } else { Sign::Positive },
            Duration::from_nanos(nanosecs.abs() as u64),
        )
    }

    pub fn zero() -> Self {
        SignedDuration::new(Sign::Positive, Duration::from_secs(0))
    }

    pub fn abs(&self) -> Duration {
        self.duration.clone()
    }
}

impl Default for SignedDuration {
    fn default() -> Self {
        SignedDuration::zero()
    }
}

impl From<Duration> for SignedDuration {
    fn from(duration: Duration) -> Self {
        Self { sign: Sign::Positive, duration }
    }
}

impl Neg for SignedDuration {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            sign: match self.sign {
                Sign::Positive => Sign::Negative,
                Sign::Negative => Sign::Positive,
            },
            duration: self.duration,
        }
    }
}


impl Add<SignedDuration> for SignedDuration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self.sign, other.sign) {
            (Sign::Positive, Sign::Positive) => Self {
                sign: Sign::Positive,
                duration: self.duration + other.duration,
            },
            (Sign::Positive, Sign::Negative) => {
                if self.duration >= other.duration {
                    Self {
                        sign: Sign::Positive,
                        duration: self.duration - other.duration,
                    }
                } else {
                    Self {
                        sign: Sign::Negative,
                        duration: other.duration - self.duration,
                    }
                }
            },
            (Sign::Negative, Sign::Positive) => {
                if other.duration >= self.duration {
                    Self {
                        sign: Sign::Positive,
                        duration: other.duration - self.duration,
                    }
                } else {
                    Self {
                        sign: Sign::Negative,
                        duration: self.duration - other.duration,
                    }
                }
            },
            (Sign::Negative, Sign::Negative) => Self {
                sign: Sign::Negative,
                duration: self.duration + other.duration,
            },
        }
    }
}

impl Sub<SignedDuration> for SignedDuration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
       self.add(other.neg())
    }
}

pub fn duration_add_assign_signed_duration(duration: &mut Duration, other: SignedDuration) {
    match other.sign {
        Sign::Positive => *duration += other.duration,
        Sign::Negative => *duration -= other.duration,
    }
}

pub fn from_f64_millis(millis: f64) -> Duration {
    Duration::from_nanos((millis * 1_000_000.0) as u64)
}

pub fn into_f64_millis(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000.0
}

pub fn from_f64_seconds(secs: f64) -> Duration {
    Duration::from_nanos((secs * 1_000_000_000.0) as u64)
}

pub fn into_f64_seconds(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000_000.0
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn test_from_f64_millis_standard_case() {
        assert_eq!(super::from_f64_millis(1.0), Duration::from_millis(1));
    }

    #[test]
    fn test_from_f64_sub_millis_case() {
        assert_eq!(super::from_f64_millis(1.5), Duration::from_micros(1500));
    }

    #[test]
    fn test_into_f64_millis_standard_case() {
        assert_eq!(super::into_f64_millis(Duration::from_millis(1)), 1.0);
    }

    #[test]
    fn test_into_f64_sub_millis_case() {
        assert_eq!(super::into_f64_millis(Duration::from_micros(1500)), 1.5);
    }

    #[test]
    fn test_from_f64_seconds_standard_case() {
        assert_eq!(
            super::from_f64_seconds(1.0),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn test_from_f64_sub_seconds_case() {
        assert_eq!(
            super::from_f64_seconds(1.5),
            Duration::from_millis(1500)
        );
    }

    #[test]
    fn test_into_f64_seconds_standard_case() {
        assert_eq!(
            super::into_f64_seconds(Duration::from_secs(1)),
            1.0
        );
    }

    #[test]
    fn test_into_f64_sub_seconds_case() {
        assert_eq!(
            super::into_f64_seconds(Duration::from_millis(1500)),
            1.5
        );
    }
}
