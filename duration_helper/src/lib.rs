use std::time::Duration;

pub fn from_f64_millis(millis: f64) -> Duration {
    Duration::from_nanos((millis * 1_000_000.0) as u64)
}

pub fn into_f64_millis(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000.0
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
}
