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
        // As of Rust nightly 4/13, Rust can return "-0" for f64, which Flash doesn't want.
        Cow::Borrowed("0")
    } else {
        // Normal number.
        Cow::Owned(n.to_string())
    }
}

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
/// Although this is easy to do on most architectures, Rust provides no standard
/// way to round in this manner (`f64::round` always rounds away from zero).
/// For more info and the below code snippet, see: https://github.com/rust-lang/rust/issues/55107
/// This also clamps out-of-range values and NaN to `i32::MIN`.
/// TODO: Investigate using SSE/wasm intrinsics for this.
pub fn round_to_even(n: f64) -> i32 {
    let k = 1.0 / f64::EPSILON;
    let a = n.abs();
    let out = if a < k { ((a + k) - k).copysign(n) } else { n };
    // Clamp out-of-range values to `i32::MIN`.
    if out.is_finite() && out >= i32::MIN.into() && out <= i32::MAX.into() {
        out as i32
    } else {
        i32::MIN
    }
}
