use std::num::Wrapping;

use super::WStr;

/// Analog of [`std::str::FromStr`], but for Ruffle's [`WStr<'_>`].
pub trait FromWStr: Sized {
    type Err;

    fn from_wstr(s: WStr<'_>) -> Result<Self, Self::Err>;
}

/// Error returned by [`Integer::from_str_radix`].
#[derive(Debug, thiserror::Error)]
#[error("failed to parse integer")]
pub struct ParseIntError(());

/// Trait implemented for all integer types that can be parsed from a [`WStr<'_>`].
pub trait Integer: FromWStr<Err = ParseIntError> {
    fn from_wstr_radix(s: WStr<'_>, radix: u32) -> Result<Self, Self::Err>;
}

impl FromWStr for f64 {
    type Err = std::num::ParseFloatError;

    fn from_wstr(s: WStr<'_>) -> Result<Self, Self::Err> {
        // TODO(moulins): avoid the utf8 conversion when we know the string can't
        // possibly represent a floating point number.
        s.to_utf8_lossy().parse()
    }
}

mod int_parse {
    use super::*;
    use std::convert::TryFrom;

    pub trait IntParse: Sized {
        const SIGNED: bool;

        fn from_digit(n: u32) -> Self;
        fn checked_add(self, n: u32) -> Option<Self>;
        fn checked_sub(self, n: u32) -> Option<Self>;
        fn checked_mul(self, n: u32) -> Option<Self>;
    }

    pub fn from_str_radix<T: IntParse>(s: WStr<'_>, radix: u32) -> Option<T> {
        assert!(
            radix >= 2 && radix <= 36,
            "from_str_radix: radix must be between 2 and 36, got {}",
            radix
        );

        let (is_neg, digits) = match s.try_get(0).map(u8::try_from) {
            Some(Ok(b'-')) => (true, s.slice(1..)),
            Some(Ok(b'+')) => (false, s.slice(1..)),
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

impl_int_parse! { u32 i32 usize }

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

impl_wrapping_int_parse! { u32 i32 usize }

macro_rules! impl_from_str_int {
    ($($ty:ty)*) => { $(
        impl Integer for $ty {
            #[inline]
            fn from_wstr_radix(s: WStr<'_>, radix: u32) -> Result<Self, ParseIntError> {
                int_parse::from_str_radix(s, radix).ok_or(ParseIntError(()))
            }
        }

        impl Integer for Wrapping<$ty> {
            #[inline]
            fn from_wstr_radix(s: WStr<'_>, radix: u32) -> Result<Self, ParseIntError> {
                int_parse::from_str_radix(s, radix).ok_or(ParseIntError(()))
            }
        }

        impl FromWStr for $ty {
            type Err = ParseIntError;
            #[inline]
            fn from_wstr(s: WStr<'_>) -> Result<Self, Self::Err> {
                int_parse::from_str_radix(s, 10).ok_or(ParseIntError(()))
            }
        }

        impl FromWStr for Wrapping<$ty> {
            type Err = ParseIntError;
            #[inline]
            fn from_wstr(s: WStr<'_>) -> Result<Self, Self::Err> {
                int_parse::from_str_radix(s, 10).ok_or(ParseIntError(()))
            }
        }
    )* }
}

impl_from_str_int! { u32 i32 usize }
