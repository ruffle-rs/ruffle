#[derive(Copy, Clone, Debug)]
pub struct ColorTransform {
    pub r_mult: f32,
    pub g_mult: f32,
    pub b_mult: f32,
    pub a_mult: f32,
    pub r_add: f32,
    pub g_add: f32,
    pub b_add: f32,
    pub a_add: f32,
}

impl From<swf::ColorTransform> for ColorTransform {
    fn from(color_transform: swf::ColorTransform) -> ColorTransform {
        ColorTransform {
            r_mult: color_transform.r_multiply,
            g_mult: color_transform.g_multiply,
            b_mult: color_transform.b_multiply,
            a_mult: color_transform.a_multiply,
            r_add: f32::from(color_transform.r_add) / 255.0,
            g_add: f32::from(color_transform.g_add) / 255.0,
            b_add: f32::from(color_transform.b_add) / 255.0,
            a_add: f32::from(color_transform.a_add) / 255.0,
        }
    }
}

impl ColorTransform {
    #[allow(clippy::float_cmp)]
    pub fn is_identity(&self) -> bool {
        self.r_mult == 1.0
            && self.g_mult == 1.0
            && self.b_mult == 1.0
            && self.a_mult == 1.0
            && self.r_add == 0.0
            && self.g_add == 0.0
            && self.b_add == 0.0
            && self.a_add == 0.0
    }
}

impl std::default::Default for ColorTransform {
    fn default() -> ColorTransform {
        ColorTransform {
            r_mult: 1.0,
            b_mult: 1.0,
            g_mult: 1.0,
            a_mult: 1.0,
            r_add: 0.0,
            b_add: 0.0,
            g_add: 0.0,
            a_add: 0.0,
        }
    }
}

impl std::ops::Mul for ColorTransform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        ColorTransform {
            r_mult: self.r_mult * rhs.r_mult,
            g_mult: self.g_mult * rhs.g_mult,
            b_mult: self.b_mult * rhs.b_mult,
            a_mult: self.a_mult * rhs.a_mult,

            r_add: self.r_mult * rhs.r_add + self.r_add,
            g_add: self.g_mult * rhs.g_add + self.g_add,
            b_add: self.b_mult * rhs.b_add + self.b_add,
            a_add: self.a_mult * rhs.a_add + self.a_add,
        }
    }
}

impl std::ops::MulAssign for ColorTransform {
    fn mul_assign(&mut self, rhs: Self) {
        *self = ColorTransform {
            r_mult: self.r_mult * rhs.r_mult,
            g_mult: self.g_mult * rhs.g_mult,
            b_mult: self.b_mult * rhs.b_mult,
            a_mult: self.a_mult * rhs.a_mult,

            r_add: self.r_mult * rhs.r_add + self.r_add,
            g_add: self.g_mult * rhs.b_add + self.g_add,
            b_add: self.b_mult * rhs.g_add + self.b_add,
            a_add: self.a_mult * rhs.a_add + self.a_add,
        }
    }
}

pub struct ColorTransformStack(Vec<ColorTransform>);

impl ColorTransformStack {
    pub fn new() -> ColorTransformStack {
        ColorTransformStack(vec![ColorTransform::default()])
    }

    pub fn push(&mut self, matrix: &ColorTransform) {
        let new_matrix = *self.color_transform() * *matrix;
        self.0.push(new_matrix);
    }

    pub fn pop(&mut self) {
        if self.0.len() <= 1 {
            panic!("Matrix stack underflow");
        }
        self.0.pop();
    }

    pub fn color_transform(&self) -> &ColorTransform {
        &self.0[self.0.len() - 1]
    }
}
