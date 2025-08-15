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
#[expect(clippy::unreadable_literal)]
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
    #[test]
    fn wrapping_u16() {
        use super::f64_to_wrapping_u16;
        assert_eq!(f64_to_wrapping_u16(0.0), 0);
        assert_eq!(f64_to_wrapping_u16(1.0), 1);
        assert_eq!(f64_to_wrapping_u16(-1.0), 65535);
        assert_eq!(f64_to_wrapping_u16(123.1), 123);
        assert_eq!(f64_to_wrapping_u16(66535.9), 999);
        assert_eq!(f64_to_wrapping_u16(-9980.7), 55556);
        assert_eq!(f64_to_wrapping_u16(-196608.0), 0);
        assert_eq!(f64_to_wrapping_u16(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u16(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u16(f64::NEG_INFINITY), 0);
    }

    #[test]

    fn wrapping_i16() {
        use super::f64_to_wrapping_i16;
        assert_eq!(f64_to_wrapping_i16(0.0), 0);
        assert_eq!(f64_to_wrapping_i16(1.0), 1);
        assert_eq!(f64_to_wrapping_i16(-1.0), -1);
        assert_eq!(f64_to_wrapping_i16(123.1), 123);
        assert_eq!(f64_to_wrapping_i16(32768.9), -32768);
        assert_eq!(f64_to_wrapping_i16(-32769.9), 32767);
        assert_eq!(f64_to_wrapping_i16(-33268.1), 32268);
        assert_eq!(f64_to_wrapping_i16(-196608.0), 0);
        assert_eq!(f64_to_wrapping_i16(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i16(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i16(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn wrapping_u32() {
        use super::f64_to_wrapping_u32;
        assert_eq!(f64_to_wrapping_u32(0.0), 0);
        assert_eq!(f64_to_wrapping_u32(1.0), 1);
        assert_eq!(f64_to_wrapping_u32(-1.0), 4294967295);
        assert_eq!(f64_to_wrapping_u32(123.1), 123);
        assert_eq!(f64_to_wrapping_u32(4294968295.9), 999);
        assert_eq!(f64_to_wrapping_u32(-4289411740.3), 5555556);
        assert_eq!(f64_to_wrapping_u32(-12884901888.0), 0);
        assert_eq!(f64_to_wrapping_u32(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u32(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u32(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn wrapping_i32() {
        use super::f64_to_wrapping_i32;
        assert_eq!(f64_to_wrapping_i32(0.0), 0);
        assert_eq!(f64_to_wrapping_i32(1.0), 1);
        assert_eq!(f64_to_wrapping_i32(-1.0), -1);
        assert_eq!(f64_to_wrapping_i32(123.1), 123);
        assert_eq!(f64_to_wrapping_i32(4294968295.9), 999);
        assert_eq!(f64_to_wrapping_i32(2147484648.3), -2147482648);
        assert_eq!(f64_to_wrapping_i32(-8589934591.2), 1);
        assert_eq!(f64_to_wrapping_i32(4294966896.1), -400);
        assert_eq!(f64_to_wrapping_i32(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i32(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i32(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn test_round_to_even() {
        use super::round_to_even;
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
