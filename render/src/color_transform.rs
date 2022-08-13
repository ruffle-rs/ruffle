use swf::Fixed8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorTransform {
    pub r_mult: Fixed8,
    pub g_mult: Fixed8,
    pub b_mult: Fixed8,
    pub a_mult: Fixed8,
    pub r_add: i16,
    pub g_add: i16,
    pub b_add: i16,
    pub a_add: i16,
}

impl From<swf::ColorTransform> for ColorTransform {
    fn from(color_transform: swf::ColorTransform) -> ColorTransform {
        ColorTransform {
            r_mult: color_transform.r_multiply,
            g_mult: color_transform.g_multiply,
            b_mult: color_transform.b_multiply,
            a_mult: color_transform.a_multiply,
            r_add: color_transform.r_add,
            g_add: color_transform.g_add,
            b_add: color_transform.b_add,
            a_add: color_transform.a_add,
        }
    }
}

impl ColorTransform {
    #[allow(clippy::float_cmp)]
    pub fn is_identity(&self) -> bool {
        self.r_mult.is_one()
            && self.g_mult.is_one()
            && self.b_mult.is_one()
            && self.a_mult.is_one()
            && self.r_add == 0
            && self.g_add == 0
            && self.b_add == 0
            && self.a_add == 0
    }

    /// Returns the multiplicative component of this color transform in RGBA order
    /// with the values normalized [0.0, 1.0].
    pub fn mult_rgba_normalized(&self) -> [f32; 4] {
        [
            self.r_mult.into(),
            self.g_mult.into(),
            self.b_mult.into(),
            self.a_mult.into(),
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
    pub fn set_mult_color(&mut self, color: &swf::Color) {
        self.r_mult = Fixed8::from_f32(f32::from(color.r) / 255.0);
        self.g_mult = Fixed8::from_f32(f32::from(color.g) / 255.0);
        self.b_mult = Fixed8::from_f32(f32::from(color.b) / 255.0);
        self.a_mult = Fixed8::from_f32(f32::from(color.a) / 255.0);
    }
}

impl Default for ColorTransform {
    fn default() -> ColorTransform {
        ColorTransform {
            r_mult: Fixed8::ONE,
            b_mult: Fixed8::ONE,
            g_mult: Fixed8::ONE,
            a_mult: Fixed8::ONE,
            r_add: 0,
            b_add: 0,
            g_add: 0,
            a_add: 0,
        }
    }
}

impl std::ops::Mul for ColorTransform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        ColorTransform {
            r_mult: self.r_mult.wrapping_mul(rhs.r_mult),
            g_mult: self.g_mult.wrapping_mul(rhs.g_mult),
            b_mult: self.b_mult.wrapping_mul(rhs.b_mult),
            a_mult: self.a_mult.wrapping_mul(rhs.a_mult),

            r_add: self
                .r_add
                .wrapping_add(self.r_mult.wrapping_mul_int(rhs.r_add)),
            g_add: self
                .g_add
                .wrapping_add(self.g_mult.wrapping_mul_int(rhs.g_add)),
            b_add: self
                .b_add
                .wrapping_add(self.b_mult.wrapping_mul_int(rhs.b_add)),
            a_add: self
                .a_add
                .wrapping_add(self.a_mult.wrapping_mul_int(rhs.a_add)),
        }
    }
}

impl std::ops::MulAssign for ColorTransform {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
