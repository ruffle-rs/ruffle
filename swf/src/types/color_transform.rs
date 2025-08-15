use crate::{Color, Fixed8};
use std::ops::{Mul, MulAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorTransform {
    pub r_multiply: Fixed8,
    pub g_multiply: Fixed8,
    pub b_multiply: Fixed8,
    pub a_multiply: Fixed8,
    pub r_add: i16,
    pub g_add: i16,
    pub b_add: i16,
    pub a_add: i16,
}

impl ColorTransform {
    pub const IDENTITY: Self = Self {
        r_multiply: Fixed8::ONE,
        b_multiply: Fixed8::ONE,
        g_multiply: Fixed8::ONE,
        a_multiply: Fixed8::ONE,
        r_add: 0,
        b_add: 0,
        g_add: 0,
        a_add: 0,
    };

    pub fn multiply_from(color: Color) -> Self {
        Self {
            r_multiply: Fixed8::from_f32(f32::from(color.r) / 255.0),
            g_multiply: Fixed8::from_f32(f32::from(color.g) / 255.0),
            b_multiply: Fixed8::from_f32(f32::from(color.b) / 255.0),
            a_multiply: Fixed8::from_f32(f32::from(color.a) / 255.0),
            ..Default::default()
        }
    }

    /// Returns the multiplicative component of this color transform in RGBA order
    /// with the values normalized [0.0, 1.0].
    pub fn mult_rgba_normalized(&self) -> [f32; 4] {
        [
            self.r_multiply.into(),
            self.g_multiply.into(),
            self.b_multiply.into(),
            self.a_multiply.into(),
        ]
    }

    /// Returns the additive component of this color transform in RGBA order
    /// with the values normalized [-1.0, 1.0].
    pub fn add_rgba_normalized(&self) -> [f32; 4] {
        [
            f32::from(self.r_add) / 255.0,
            f32::from(self.g_add) / 255.0,
            f32::from(self.b_add) / 255.0,
            f32::from(self.a_add) / 255.0,
        ]
    }

    /// Sets the multiplicate component of this color transform.
    pub fn set_mult_color(&mut self, color: &Color) {
        self.r_multiply = Fixed8::from_f32(f32::from(color.r) / 255.0);
        self.g_multiply = Fixed8::from_f32(f32::from(color.g) / 255.0);
        self.b_multiply = Fixed8::from_f32(f32::from(color.b) / 255.0);
        self.a_multiply = Fixed8::from_f32(f32::from(color.a) / 255.0);
    }
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for ColorTransform {
    type Output = ColorTransform;

    fn mul(self, rhs: Self) -> Self::Output {
        ColorTransform {
            r_multiply: self.r_multiply.wrapping_mul(rhs.r_multiply),
            g_multiply: self.g_multiply.wrapping_mul(rhs.g_multiply),
            b_multiply: self.b_multiply.wrapping_mul(rhs.b_multiply),
            a_multiply: self.a_multiply.wrapping_mul(rhs.a_multiply),
            r_add: self
                .r_add
                .wrapping_add(self.r_multiply.wrapping_mul_int(rhs.r_add)),
            g_add: self
                .g_add
                .wrapping_add(self.g_multiply.wrapping_mul_int(rhs.g_add)),
            b_add: self
                .b_add
                .wrapping_add(self.b_multiply.wrapping_mul_int(rhs.b_add)),
            a_add: self
                .a_add
                .wrapping_add(self.a_multiply.wrapping_mul_int(rhs.a_add)),
        }
    }
}

impl MulAssign for ColorTransform {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Color> for &ColorTransform {
    type Output = Color;

    fn mul(self, mut color: Color) -> Color {
        if color.a > 0 {
            color.r = self
                .r_multiply
                .mul_int(i16::from(color.r))
                .saturating_add(self.r_add)
                .clamp(0, 255) as u8;
            color.g = self
                .g_multiply
                .mul_int(i16::from(color.g))
                .saturating_add(self.g_add)
                .clamp(0, 255) as u8;
            color.b = self
                .b_multiply
                .mul_int(i16::from(color.b))
                .saturating_add(self.b_add)
                .clamp(0, 255) as u8;
            color.a = self
                .a_multiply
                .mul_int(i16::from(color.a))
                .saturating_add(self.a_add)
                .clamp(0, 255) as u8;
        }
        color
    }
}
