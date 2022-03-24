use core::fmt;
use core::num::Wrapping;

use super::WStr;

/// Analog of [`std::str::FromStr`], but for Ruffle's [`&WStr`].
pub trait FromWStr: Sized {
    type Err;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err>;
}

/// Error returned by [`Integer::from_str_radix`].
#[derive(Debug)]
pub struct ParseNumError(());

impl fmt::Display for ParseNumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse integer")
    }
}

/// Trait implemented for all integer types that can be parsed from a [`WStr`].
pub trait Integer: FromWStr<Err = ParseNumError> {
    fn from_wstr_radix(s: &WStr, radix: u32) -> Result<Self, Self::Err>;
}

fn parse_special_floats(s: &WStr) -> Option<f64> {
    let nan = WStr::from_units(b"NaN");
    let inf = WStr::from_units(b"inf");

    let slice = match s.len() {
        3 if s == nan => return Some(f64::NAN),
        3 if s == inf => return Some(f64::INFINITY),
        4 => &s[1..],
        _ => return None,
    };

    let is_nan = if slice == nan {
        true
    } else if slice == inf {
        false
    } else {
        return None;
    };
    let is_neg = match u8::try_from(s.at(0)) {
        Ok(b'+') => false,
        Ok(b'-') => true,
        _ => return None,
    };

    Some(match (is_nan, is_neg) {
        (false, false) => f64::INFINITY,
        (false, true) => f64::NEG_INFINITY,
        (true, _) => f64::NAN,
    })
}

impl FromWStr for f64 {
    type Err = ParseNumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if let Some(f) = parse_special_floats(s) {
            return Ok(f);
        }

        // Early-reject strings with non-float chars to avoid the utf8 conversion.
        let is_valid = s.iter().all(|c| {
            if let Ok(c) = u8::try_from(c) {
                matches!(c, b'0'..=b'9' | b'.' | b'+' | b'-' | b'e' | b'E')
            } else {
                false
            }
        });

        if is_valid {
            s.to_utf8_lossy().parse().map_err(|_| ParseNumError(()))
        } else {
            Err(ParseNumError(()))
        }
    }
}

mod int_parse {
    use super::*;

    pub trait IntParse: Sized {
        const SIGNED: bool;

        fn from_digit(n: u32) -> Self;
        fn checked_add(self, n: u32) -> Option<Self>;
        fn checked_sub(self, n: u32) -> Option<Self>;
        fn checked_mul(self, n: u32) -> Option<Self>;
    }

    pub fn from_wstr_radix<T: IntParse>(s: &WStr, radix: u32) -> Option<T> {
        assert!(
            radix >= 2 && radix <= 36,
            "from_str_radix: radix must be between 2 and 36, got {}",
            radix
        );

        let (is_neg, digits) = match s.get(0).map(u8::try_from) {
            Some(Ok(b'-')) => (true, &s[1..]),
            Some(Ok(b'+')) => (false, &s[1..]),
            Some(_) => (false, s),
            None => return None,
        };

        if is_neg && !T::SIGNED {
            return None;
        }

        digits.iter().try_fold(T::from_digit(0), |num, c| {
            let byte = u8::try_from(c).ok()?;
            let digit = (byte as char).to_digit(radix)?;
            let num = num.checked_mul(radix)?;
            if is_neg {
                num.checked_sub(digit)
            } else {
                num.checked_add(digit)
            }
        })
    }
}

macro_rules! impl_int_parse {
    ($($ty:ty)*) => { $(
        impl int_parse::IntParse for $ty {
            #[allow(unused_comparisons)]
            const SIGNED: bool = <$ty>::MIN < 0;

            #[inline]
            fn from_digit(n: u32) -> Self {
                n as $ty
            }

            #[inline]
            fn checked_add(self, n: u32) -> Option<Self> {
                <$ty>::checked_add(self, n as $ty)
            }

            #[inline]
            fn checked_sub(self, n: u32) -> Option<Self> {
                <$ty>::checked_sub(self, n as $ty)
            }

            #[inline]
            fn checked_mul(self, n: u32) -> Option<Self> {
                <$ty>::checked_mul(self, n as $ty)
            }
        }
    )* }
}

impl_int_parse! { u8 u32 i32 usize }

macro_rules! impl_wrapping_int_parse {
    ($($ty:ty)*) => { $(
        impl int_parse::IntParse for Wrapping<$ty> {
            #[allow(unused_comparisons)]
            const SIGNED: bool = <$ty>::MIN < 0;

            #[inline]
            fn from_digit(n: u32) -> Self {
                Self(n as $ty)
            }

            #[inline]
            fn checked_add(self, n: u32) -> Option<Self> {
                Some(self + Self::from_digit(n))
            }

            #[inline]
            fn checked_sub(self, n: u32) -> Option<Self> {
                Some(self - Self::from_digit(n))
            }

            #[inline]
            fn checked_mul(self, n: u32) -> Option<Self> {
                Some(self * Self::from_digit(n))
            }
        }
    )* }
}

impl_wrapping_int_parse! { u8 u32 i32 usize }

macro_rules! impl_from_str_int {
    ($($ty:ty)*) => { $(
        impl Integer for $ty {
            #[inline]
            fn from_wstr_radix(s: &WStr, radix: u32) -> Result<Self, ParseNumError> {
                int_parse::from_wstr_radix(s, radix).ok_or(ParseNumError(()))
            }
        }

        impl Integer for Wrapping<$ty> {
            #[inline]
            fn from_wstr_radix(s: &WStr, radix: u32) -> Result<Self, ParseNumError> {
                int_parse::from_wstr_radix(s, radix).ok_or(ParseNumError(()))
            }
        }

        impl FromWStr for $ty {
            type Err = ParseNumError;
            #[inline]
            fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
                int_parse::from_wstr_radix(s, 10).ok_or(ParseNumError(()))
            }
        }

        impl FromWStr for Wrapping<$ty> {
            type Err = ParseNumError;
            #[inline]
            fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
                int_parse::from_wstr_radix(s, 10).ok_or(ParseNumError(()))
            }
        }
    )* }
}

impl_from_str_int! { u8 u32 i32 usize }
