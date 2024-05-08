//! ECMA-262 compliant numerical conversions

/// Converts an `f64` to a `u8` with ECMAScript `ToUInt8` wrapping behavior.
/// The value will be wrapped modulo 2^8.
pub fn f64_to_wrapping_u8(n: f64) -> u8 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(256.0) as u8
    }
}

/// Converts an `f64` to a `u16` with ECMAScript `ToUInt16` wrapping behavior.
/// The value will be wrapped modulo 2^16.
pub fn f64_to_wrapping_u16(n: f64) -> u16 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(65536.0) as u16
    }
}

/// Converts an `f64` to an `i16` with ECMAScript wrapping behavior.
/// The value will be wrapped in the range [-2^15, 2^15).
pub fn f64_to_wrapping_i16(n: f64) -> i16 {
    f64_to_wrapping_u16(n) as i16
}

/// Converts an `f64` to a `u32` with ECMAScript `ToUInt32` wrapping behavior.
/// The value will be wrapped modulo 2^32.
#[allow(clippy::unreadable_literal)]
pub fn f64_to_wrapping_u32(n: f64) -> u32 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(4294967296.0) as u32
    }
}

/// Converts an `f64` to an `i32` with ECMAScript `ToInt32` wrapping behavior.
/// The value will be wrapped in the range [-2^31, 2^31).
pub fn f64_to_wrapping_i32(n: f64) -> i32 {
    f64_to_wrapping_u32(n) as i32
}

/// Implements the IEEE-754 "Round to nearest, ties to even" rounding rule.
/// (e.g., both 1.5 and 2.5 will round to 2).
/// This also clamps out-of-range values and NaN to `i32::MIN`.
pub fn round_to_even(n: f64) -> i32 {
    let out = n.round_ties_even();
    // Clamp out-of-range values to `i32::MIN`.
    if out.is_finite() && out <= i32::MAX.into() {
        out as i32
    } else {
        i32::MIN
    }
}

#[cfg(test)]
mod test {
    use super::round_to_even;

    #[test]
    fn test_round_to_even() {
        assert_eq!(round_to_even(0.0), 0);
        assert_eq!(round_to_even(2.0), 2);
        assert_eq!(round_to_even(2.1), 2);
        assert_eq!(round_to_even(2.5), 2);
        assert_eq!(round_to_even(2.9), 3);
        assert_eq!(round_to_even(3.0), 3);
        assert_eq!(round_to_even(3.1), 3);
        assert_eq!(round_to_even(3.5), 4);
        assert_eq!(round_to_even(3.9), 4);
        assert_eq!(round_to_even(4.0), 4);
        assert_eq!(round_to_even(-2.0), -2);
        assert_eq!(round_to_even(-2.1), -2);
        assert_eq!(round_to_even(-2.5), -2);
        assert_eq!(round_to_even(-2.9), -3);
        assert_eq!(round_to_even(-3.0), -3);
        assert_eq!(round_to_even(-3.1), -3);
        assert_eq!(round_to_even(-3.5), -4);
        assert_eq!(round_to_even(-3.9), -4);
        assert_eq!(round_to_even(-4.0), -4);
        assert_eq!(round_to_even(f64::NAN), i32::MIN);
        assert_eq!(round_to_even(f64::INFINITY), i32::MIN);
        assert_eq!(round_to_even(f64::NEG_INFINITY), i32::MIN);
        assert_eq!(round_to_even(-2147483648f64), i32::MIN);
        assert_eq!(round_to_even(-2247483648f64), i32::MIN);
        assert_eq!(round_to_even(2147483647f64), i32::MAX);
        assert_eq!(round_to_even(2247483647f64), i32::MIN);
    }
}
