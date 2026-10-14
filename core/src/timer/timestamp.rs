use gc_arena::Collect;
use std::ops::{Add, AddAssign, Sub};
use std::time::Duration;

/// A point in time used by timers, measured in microseconds from when the
/// player started.
///
/// Arithmetic saturates, so a timestamp never goes past `u64::MAX` microseconds
/// (over 500,000 years) and never wraps around.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Collect)]
#[collect(require_static)]
pub struct TimerTimestamp(u64);

impl TimerTimestamp {
    pub const ZERO: Self = Self(0);
}

impl Add<Duration> for TimerTimestamp {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        let micros = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        Self(self.0.saturating_add(micros))
    }
}

impl AddAssign<Duration> for TimerTimestamp {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub for TimerTimestamp {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        Duration::from_micros(self.0.saturating_sub(other.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_duration() {
        let timestamp = TimerTimestamp::ZERO + Duration::from_millis(10);
        assert_eq!(timestamp, TimerTimestamp(10_000));
        assert_eq!(timestamp + Duration::from_micros(5), TimerTimestamp(10_005));
    }

    #[test]
    fn add_duration_truncates_to_micros() {
        let timestamp = TimerTimestamp::ZERO + Duration::from_nanos(1_999);
        assert_eq!(timestamp, TimerTimestamp(1));
    }

    #[test]
    fn add_assign_duration() {
        let mut timestamp = TimerTimestamp(7);
        timestamp += Duration::from_micros(3);
        assert_eq!(timestamp, TimerTimestamp(10));
    }

    #[test]
    fn add_duration_saturates() {
        let timestamp = TimerTimestamp(u64::MAX - 1) + Duration::from_micros(2);
        assert_eq!(timestamp, TimerTimestamp(u64::MAX));
        assert_eq!(TimerTimestamp(1) + Duration::MAX, TimerTimestamp(u64::MAX));
    }

    #[test]
    fn sub_timestamps() {
        let duration = TimerTimestamp(10_005) - TimerTimestamp(5);
        assert_eq!(duration, Duration::from_millis(10));
        assert_eq!(TimerTimestamp(5) - TimerTimestamp(5), Duration::ZERO);
    }

    #[test]
    fn sub_timestamps_saturates() {
        let duration = TimerTimestamp(1) - TimerTimestamp(u64::MAX);
        assert_eq!(duration, Duration::ZERO);
    }

    #[test]
    fn ordering() {
        let earlier = TimerTimestamp::ZERO + Duration::from_micros(1);
        let later = earlier + Duration::from_micros(1);
        assert!(TimerTimestamp::ZERO < earlier);
        assert!(earlier < later);
    }
}
