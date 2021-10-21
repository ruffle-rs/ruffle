//! Fixed-point types.
//!
//! `Fixed8` is an 8.8 signed fixed-point number.
//! `Fixed16` is a 16.16 signed fixed-point number.
//!
//! This is not meant to be a fully general fixed-point library, but instead focused on the needs of Ruffle/Flash.
//! No rounding adjustments are done. All calculations are truncated to match Flash's behavior.
//!
//! Use the `From` trait to convert losslessly from an integer to fixed-point.
//! Use `from_f32`/`from_f64` methods to convert from float to fixed-point.
//! Extra precision will be truncated, and out-of-range values are saturated.

use std::ops::*;

macro_rules! define_fixed {
    (
        $type_name:ident, $underlying_type:path, $intermediate_type:path, $frac_bits:literal,
        from_int($($from_type:path),*),
        into_float($($into_type:path),*)
    ) => {
        /// A signed fixed-point value with $frac_bits bits.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $type_name($underlying_type);

        /// A signed fixed-point type.
        impl $type_name {
            /// The number of integer bits.
            pub const INTEGER_BITS: u8 =
                (std::mem::size_of::<$underlying_type>() as u8) * 8 - $frac_bits;

            /// The number of fractional bits.
            pub const FRACTIONAL_BITS: u8 = $frac_bits;

            /// The fixed-point value representing `0.0`.
            pub const ZERO: Self = Self(0);

            /// The fixed-point value representing `1.0`.
            pub const ONE: Self = Self(1 << Self::FRACTIONAL_BITS);

            /// The minimum representable value of this type.
            pub const MIN: Self = Self(<$underlying_type>::MIN);

            /// The maximum representable value of this type.
            pub const MAX: Self = Self(<$underlying_type>::MAX);

            /// Returns the fixed-point value with the same bit-representation as the given value.
            #[inline]
            pub const fn from_bits(n: $underlying_type) -> Self {
                Self(n)
            }

            #[inline]
            pub const fn get(self) -> $underlying_type {
                self.0
            }

            /// Converts an `f32` floating-point value to fixed point.
            ///
            /// This conversion may be lossy, with behavior like standard Rust float-to-int casting.
            /// Extra precision will be truncated, and the result will be saturated if it doesn't
            /// fit in the underlying type's range. `NaN` returns `0.0`.
            #[inline]
            pub fn from_f32(n: f32) -> Self {
                Self((n * (1 << Self::FRACTIONAL_BITS) as f32) as $underlying_type)
            }

            /// Converts an `f64` floating-point value to fixed-point.
            ///
            /// This conversion may be lossy, with behavior like standard Rust float-to-int casting.
            /// Extra precision will be truncated, and the result will be saturated if it doesn't
            /// fit in the underlying type's range. `NaN` returns `0.0`.
            #[inline]
            pub fn from_f64(n: f64) -> Self {
                Self((n * (1 << Self::FRACTIONAL_BITS) as f64) as $underlying_type)
            }

            /// Converts this fixed-point value to `f32` floating-point.
            ///
            /// This conversion may be lossy if `f32` does not have enough precision
            /// to represent the fixed-point value.
            ///
            /// Use `From` to ensure that the conversion is lossless at compile-time.
            #[inline]
            pub fn to_f32(self) -> f32 {
                self.0 as f32 / (1 << Self::FRACTIONAL_BITS) as f32
            }

            /// Converts this fixed-point value to `f64` floating-point.
            ///
            /// This conversion may be lossy if `f64` does not have enough precision
            /// to represent the fixed-point value.
            ///
            /// Use `From` to ensure that the conversion is lossless at compile-time.
            #[inline]
            pub fn to_f64(self) -> f64 {
                self.0 as f64 / (1 << Self::FRACTIONAL_BITS) as f64
            }

            /// Returns `true` if this is equal to `0.0`.
            #[inline]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if this is equal to `1.0`.
            #[inline]
            pub const fn is_one(self) -> bool {
                self.0 == 1 << Self::FRACTIONAL_BITS
            }

            /// Multiplies this fixed-point by an integer, returning the integer result.
            /// The result uses full range of the integer. The fractional bits will be truncated.
            #[inline]
            pub fn mul_int(self, other: $underlying_type) -> $underlying_type {
                let n = (<$intermediate_type>::from(self.0) * <$intermediate_type>::from(other))
                    >> Self::FRACTIONAL_BITS;
                if cfg!(debug_assertions) {
                    // Check for overflow.
                    <$underlying_type>::try_from(n).expect("Attempted to multiply with overflow")
                } else {
                    n as $underlying_type
                }
            }

            /// Wrapping (modular) negation. Computes -self, wrapping around at the boundary of the type.
            /// -Self::MIN is the only case where wrapping occurs.
            #[inline]
            pub fn wrapping_neg(self) -> Self {
                Self(self.0.wrapping_neg())
            }

            /// Wrapping (modular) addition. Computes self + rhs, wrapping around at the boundary of the type.
            #[inline]
            pub fn wrapping_add(self, other: Self) -> Self {
                Self(self.0.wrapping_add(other.0))
            }

            /// Wrapping (modular) subtraction. Computes self - rhs, wrapping around at the boundary of the type.
            #[inline]
            pub fn wrapping_sub(self, other: Self) -> Self {
                Self(self.0.wrapping_sub(other.0))
            }

            /// Wrapping (modular) multiplication. Computes self * rhs, wrapping around at the boundary of the type.
            #[inline]
            pub fn wrapping_mul(self, other: Self) -> Self {
                let n = <$intermediate_type>::from(self.0)
                    .wrapping_mul(<$intermediate_type>::from(other.0))
                    >> Self::FRACTIONAL_BITS;
                Self(n as $underlying_type)
            }

            /// Wrapping (modular) division. Computes self / rhs, wrapping around at the boundary of the type.
            #[inline]
            pub fn wrapping_div(self, other: Self) -> Self {
                let n = (<$intermediate_type>::from(self.0) << Self::FRACTIONAL_BITS).wrapping_div(<$intermediate_type>::from(other.0));
                Self(n as $underlying_type)
            }

            /// Wrapping (modular) multiplication.
            /// Multiplies this fixed-point by an integer, returning the integer result.
            /// The result will use the full size of the integer. The fractional bits will be truncated.
            #[inline]
            pub fn wrapping_mul_int(self, other: $underlying_type) -> $underlying_type {
                let n = (<$intermediate_type>::from(self.0)
                    .wrapping_mul(<$intermediate_type>::from(other)))
                    >> Self::FRACTIONAL_BITS;
                n as $underlying_type
            }
        }

        impl Default for $type_name {
            /// Returns the default value of `0.0`.
            #[inline]
            fn default() -> Self {
                Self(0)
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.to_f64())
            }
        }

        impl Neg for $type_name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        // fixed + fixed
        impl Add for $type_name {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }

        // fixed += fixed
        impl AddAssign for $type_name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other
            }
        }

        // fixed - fixed
        impl Sub for $type_name {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }
        }

        // fixed -= fixed
        impl SubAssign for $type_name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other
            }
        }

        // fixed * fixed
        impl Mul for $type_name {
            type Output = Self;

            #[inline]
            fn mul(self, other: Self) -> Self {
                let n = <$intermediate_type>::from(self.0) * <$intermediate_type>::from(other.0)
                    >> Self::FRACTIONAL_BITS;
                if cfg!(debug_assertions) {
                    Self(<$underlying_type>::try_from(n).expect("Attempted to multiply with overflow"))
                } else {
                    Self(n as $underlying_type)
                }
            }
        }

        // fixed *= fixed
        impl MulAssign for $type_name {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other
            }
        }

        // fixed * int
        impl Mul<$underlying_type> for $type_name {
            type Output = Self;

            #[inline]
            fn mul(self, other: $underlying_type) -> Self {
                Self(self.0 * other)
            }
        }

        // fixed *= int
        impl MulAssign<$underlying_type> for $type_name {
            #[inline]
            fn mul_assign(&mut self, other: $underlying_type) {
                *self = *self * other
            }
        }

        // int * fixed
        impl Mul<$type_name> for $underlying_type {
            type Output = $type_name;

            #[inline]
            fn mul(self, other: $type_name) -> $type_name {
                other * self
            }
        }

        // fixed / fixed
        impl Div for $type_name {
            type Output = Self;

            #[inline]
            fn div(self, other: Self) -> Self {
                let n = ((<$intermediate_type>::from(self.0) << Self::FRACTIONAL_BITS)
                    / <$intermediate_type>::from(other.0));
                if cfg!(debug_assertions) {
                    Self(<$underlying_type>::try_from(n).expect("Attempted to divide with overflow"))
                } else {
                    Self(n as $underlying_type)
                }
            }
        }

        // fixed /= fixed
        impl DivAssign for $type_name {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other
            }
        }

        // fixed / int
        impl Div<$underlying_type> for $type_name {
            type Output = Self;

            #[inline]
            fn div(self, other: $underlying_type) -> Self {
                Self(self.0 / other)
            }
        }

        // fixed /= int
        impl DivAssign<$underlying_type> for $type_name {
            #[inline]
            fn div_assign(&mut self, other: $underlying_type) {
                *self = *self / other
            }
        }

        // smaller int -> fixed cast
        $(
            impl From<$from_type> for $type_name {
                #[inline]
                fn from(n: $from_type) -> Self {
                    Self(<$underlying_type>::from(n) << <$underlying_type>::from(Self::FRACTIONAL_BITS))
                }
            }
        )*

        // fixed -> larger float cast
        $(
            impl From<$type_name> for $into_type {
                #[inline]
                fn from(n: $type_name) -> $into_type {
                    n.0 as $into_type / (1 << <$type_name>::FRACTIONAL_BITS) as $into_type
                }
            }
        )*
    };
}

define_fixed!(Fixed8, i16, i32, 8, from_int(i8), into_float(f32, f64));
define_fixed!(Fixed16, i32, i64, 16, from_int(i8, i16), into_float(f64));

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::eq_op)]
pub mod tests {
    use super::*;

    #[test]
    fn from_int() {
        assert_eq!(Fixed8::from(0).get(), 0x0);
        assert_eq!(Fixed8::from(1).get(), 0x01_00);
        assert_eq!(Fixed8::from(-1).get(), 0xff_00_u16 as i16);
        assert_eq!(Fixed8::from(54).get(), 0x36_00);
        assert_eq!(Fixed8::from(-84).get(), 0xac_00_u16 as i16);
        assert_eq!(Fixed8::from(127).get(), 0x7f_00);
        assert_eq!(Fixed8::from(-128).get(), 0x80_00_u16 as i16);
    }

    #[test]
    fn from_float() {
        assert_eq!(Fixed8::from_f64(0.0).get(), 0x0);
        assert_eq!(Fixed8::from_f64(0.5).get(), 0x00_80);
        assert_eq!(Fixed8::from_f64(-12.01171875).get(), 0xf3_fd_u16 as i16);
        assert_eq!(Fixed8::from_f64(127.99609375).get(), 0x7f_ff_u16 as i16);
        assert_eq!(Fixed8::from_f64(-0.00390625).get(), 0xff_ff_u16 as i16);
        assert_eq!(Fixed8::from_f64(-128.0).get(), 0x80_00_u16 as i16);

        // Out of bounds
        assert_eq!(
            Fixed8::from_f64(f64::NEG_INFINITY).get(),
            0x80_00_u16 as i16
        );
        assert_eq!(Fixed8::from_f64(f64::INFINITY).get(), 0x7f_ff);
        assert_eq!(Fixed8::from_f64(f64::NAN).get(), 0x00_00);

        // Truncated (rounds toward zero)
        assert_eq!(Fixed8::from_f64(0.002).get(), 0x0);
        assert_eq!(Fixed8::from_f64(64.004).get(), 0x40_01);
        assert_eq!(Fixed8::from_f64(-64.004).get(), 0xbf_ff_u16 as i16);
    }

    #[test]
    fn to_float() {
        assert_eq!(Fixed8::from_bits(0x0).to_f64(), 0.0);
        assert_eq!(Fixed8::from_bits(0x01_00).to_f64(), 1.0);
        assert_eq!(Fixed8::from_bits(0x00_80).to_f64(), 0.5);
        assert_eq!(Fixed8::from_bits(0xf3_fd_u16 as i16).to_f64(), -12.01171875);
        assert_eq!(Fixed8::from_bits(0x7f_ff).to_f64(), 127.99609375);
        assert_eq!(Fixed8::from_bits(0xff_ff_u16 as i16).to_f64(), -0.00390625);
        assert_eq!(Fixed8::from_bits(0x80_00_u16 as i16).to_f64(), -128.0);
    }

    #[test]
    fn display() {
        assert_eq!(Fixed8::ZERO.to_string(), "0");
        assert_eq!(Fixed8::ONE.to_string(), "1");
        assert_eq!(Fixed8::from(5).to_string(), "5");
        assert_eq!(Fixed8::from_f64(-3.5).to_string(), "-3.5");
        assert_eq!(Fixed8::from_f64(127.99609375).to_string(), "127.99609375");
    }

    #[test]
    fn neg() {
        assert_eq!(-Fixed8::from(0), Fixed8::from(0));
        assert_eq!(-Fixed8::from(1), Fixed8::from(-1));
        assert_eq!(-Fixed8::from(-2), Fixed8::from(2));
        assert_eq!(-Fixed8::from_f64(33.5), Fixed8::from_f64(-33.5));
        assert_eq!(
            -Fixed8::from_f64(127.99609375),
            Fixed8::from_f64(-127.99609375)
        );
    }

    #[test]
    fn wrapping_neg() {
        assert_eq!(Fixed8::from(-128).wrapping_neg(), Fixed8::from(-128));
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn neg_overflow() {
        let _ = -Fixed8::from(-128);
    }

    #[test]
    fn add() {
        assert_eq!(Fixed8::ZERO + Fixed8::ZERO, Fixed8::ZERO);
        assert_eq!(Fixed8::ZERO + Fixed8::ONE, Fixed8::ONE);
        assert_eq!(Fixed8::ONE + Fixed8::ZERO, Fixed8::ONE);
        assert_eq!(Fixed8::from(7) + Fixed8::from(5), Fixed8::from(12));
        assert_eq!(
            Fixed8::from_f64(1.75) + Fixed8::from_f64(7.25),
            Fixed8::from_f64(9.0),
        );
        assert_eq!(
            Fixed8::from_f64(123.5) + Fixed8::from_f64(-1.0),
            Fixed8::from_f64(122.5),
        );
        assert_eq!(
            Fixed8::from_f64(-64.5) + Fixed8::from_f64(-5.125),
            Fixed8::from_f64(-69.625),
        );

        let mut n = Fixed8::from_f64(126.0);
        n += Fixed8::from_f64(1.5);
        assert_eq!(n, Fixed8::from_f64(127.5))
    }

    #[test]
    fn add_wrapped() {
        assert_eq!(
            Fixed8::from(-128).wrapping_add(Fixed8::from(-1)),
            Fixed8::from(127)
        );
        assert_eq!(
            Fixed8::from(127).wrapping_add(Fixed8::from(1)),
            Fixed8::from(-128)
        );
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn add_overflow() {
        let _ = Fixed8::from(-128) + Fixed8::from(-1);
    }

    #[test]
    fn sub() {
        assert_eq!(Fixed8::ZERO - Fixed8::ZERO, Fixed8::ZERO);
        assert_eq!(Fixed8::ONE - Fixed8::ZERO, Fixed8::ONE);
        assert_eq!(Fixed8::ZERO - Fixed8::ONE, Fixed8::from(-1));
        assert_eq!(Fixed8::from(7) - Fixed8::from(5), Fixed8::from(2));
        assert_eq!(
            Fixed8::from_f64(1.75) - Fixed8::from_f64(7.25),
            Fixed8::from_f64(-5.5),
        );
        assert_eq!(
            Fixed8::from_f64(123.5) - Fixed8::from_f64(-1.0),
            Fixed8::from_f64(124.5),
        );
        assert_eq!(
            Fixed8::from_f64(-64.5) - Fixed8::from_f64(-5.125),
            Fixed8::from_f64(-59.375),
        );

        let mut n = Fixed8::from_f64(126.0);
        n -= Fixed8::from_f64(1.5);
        assert_eq!(n, Fixed8::from_f64(124.5))
    }

    #[test]
    fn sub_wrapped() {
        assert_eq!(
            Fixed8::from(-128).wrapping_sub(Fixed8::from(1)),
            Fixed8::from(127)
        );
        assert_eq!(
            Fixed8::from(127).wrapping_sub(Fixed8::from(-1)),
            Fixed8::from(-128)
        );
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn sub_overflow() {
        let _ = Fixed8::from(-128) - Fixed8::from(1);
    }

    #[test]
    fn mul() {
        assert_eq!(Fixed8::ZERO * Fixed8::ZERO, Fixed8::ZERO);
        assert_eq!(Fixed8::ONE * Fixed8::ZERO, Fixed8::ZERO);
        assert_eq!(Fixed8::from(7) * Fixed8::from(5), Fixed8::from(35));
        assert_eq!(Fixed8::from(5) * Fixed8::from(7), Fixed8::from(35));
        assert_eq!(
            Fixed8::from_f64(1.75) * Fixed8::from_f64(7.25),
            Fixed8::from_f64(12.6875),
        );
        assert_eq!(
            Fixed8::from_f64(-1.75) * Fixed8::from_f64(7.25),
            Fixed8::from_f64(-12.6875),
        );

        // Result is truncated (rounds toward negative infinity).
        assert_eq!(
            Fixed8::from_f64(-5.03125) * Fixed8::from_f64(-5.03125),
            Fixed8::from_f64(25.3125),
        );
        assert_eq!(
            Fixed8::from_f64(5.03125) * Fixed8::from_f64(-5.03125),
            Fixed8::from_f64(-25.31640625),
        );

        let mut n = Fixed8::from_f64(126.0);
        n -= Fixed8::from_f64(1.5);
        assert_eq!(n, Fixed8::from_f64(124.5))
    }

    #[test]
    fn mul_wrapped() {
        assert_eq!(
            Fixed8::from(-128).wrapping_sub(Fixed8::from(1)),
            Fixed8::from(127)
        );
        assert_eq!(
            Fixed8::from(127).wrapping_sub(Fixed8::from(-1)),
            Fixed8::from(-128)
        );
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn mul_overflow() {
        let _ = Fixed8::from(64) * Fixed8::from(64);
    }

    #[test]
    fn mul_int() {
        assert_eq!(Fixed8::from_f64(1.5).mul_int(2), 3);
        assert_eq!(Fixed8::from_f64(-1.5).mul_int(2), -3);
        assert_eq!(Fixed8::from_f64(1.5).mul_int(-2), -3);
        assert_eq!(Fixed8::from_f64(-1.5).mul_int(-2), 3);

        // Verify that 16-bit value is calculated.
        assert_eq!(Fixed8::from_f64(5.5).mul_int(126), 693);

        // Truncate (round towards negative infinity).
        assert_eq!(Fixed8::from_f64(3.75).mul_int(101), 378);
        assert_eq!(Fixed8::from_f64(-3.75).mul_int(101), -379);
    }

    #[test]
    fn mul_int_wrapped() {
        assert_eq!(Fixed8::from_f64(127.5).wrapping_mul_int(30001), 24039);
        assert_eq!(Fixed8::from_f64(-127.5).wrapping_mul_int(30001), -24040);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn mul_int_overflow() {
        let _ = Fixed8::from_f64(127.5).mul_int(30001);
    }

    #[test]
    fn div() {
        assert_eq!(Fixed8::ZERO / Fixed8::ONE, Fixed8::ZERO);
        assert_eq!(Fixed8::from(4) / Fixed8::from(2), Fixed8::from(2));
        assert_eq!(Fixed8::from(2) / Fixed8::from(4), Fixed8::from_f64(0.5));
        assert_eq!(
            Fixed8::from_f64(7.5) / Fixed8::from_f64(0.5),
            Fixed8::from_f64(15.0),
        );
        assert_eq!(
            Fixed8::from_f64(68.75) / Fixed8::from_f64(-12.5),
            Fixed8::from_f64(-5.5),
        );

        // Result is truncated (rounds toward zero).
        assert_eq!(
            Fixed8::from_f64(-84.25) / Fixed8::from_f64(-5.03125),
            Fixed8::from_f64(16.7421875),
        );
        assert_eq!(
            Fixed8::from_f64(84.25) / Fixed8::from_f64(-5.03125),
            Fixed8::from_f64(-16.7421875),
        );

        let mut n = Fixed8::from_f64(126.0);
        n /= Fixed8::from_f64(1.5);
        assert_eq!(n, Fixed8::from_f64(84.0))
    }

    #[test]
    fn div_wrapped() {
        assert_eq!(
            Fixed8::from(-128).wrapping_div(Fixed8::from(-1)),
            Fixed8::from(-128)
        );
        assert_eq!(
            Fixed8::from(127).wrapping_div(Fixed8::from_f64(0.5)),
            Fixed8::from(-2)
        );
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn div_overflow() {
        let _ = Fixed8::from(-128) / Fixed8::from(-1);
    }

    #[test]
    #[should_panic]
    fn div_by_zero() {
        let _ = Fixed8::ONE / Fixed8::ZERO;
    }
}
