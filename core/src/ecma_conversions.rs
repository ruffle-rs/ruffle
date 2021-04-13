//! ECMA-262 compliant numerical conversions

use std::borrow::Cow;

/// Converts an `f64` to a String with (hopefully) the same output as Flash.
/// For example, NAN returns `"NaN"`, and infinity returns `"Infinity"`.
pub fn f64_to_string(n: f64) -> Cow<'static, str> {
    if n.is_nan() {
        Cow::Borrowed("NaN")
    } else if n == f64::INFINITY {
        Cow::Borrowed("Infinity")
    } else if n == f64::NEG_INFINITY {
        Cow::Borrowed("-Infinity")
    } else if n != 0.0 && (n.abs() >= 1e15 || n.abs() < 1e-5) {
        // Exponential notation.
        // Cheating a bit here; Flash always put a sign in front of the exponent, e.g. 1e+15.
        // Can't do this with rust format params, so shove it in there manually.
        let mut s = format!("{:e}", n);
        if let Some(i) = s.find('e') {
            if s.as_bytes().get(i + 1) != Some(&b'-') {
                s.insert(i + 1, '+');
            }
        }
        Cow::Owned(s)
    } else if n == 0.0 {
        // As of Rust nightly 4/13, Rust can returns an unwated "-0" for f64, which Flash doesn't want.
        Cow::Borrowed("0")
    } else {
        // Normal number.
        Cow::Owned(n.to_string())
    }
}

/// Converts an `f64` to an `u16` with ECMAScript `ToUInt16` wrapping behavior.
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

/// Converts an `f64` to an `u32` with ECMAScript `ToUInt32` wrapping behavior.
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
