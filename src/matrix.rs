#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl From<swf::Matrix> for Matrix {
    fn from(matrix: swf::Matrix) -> Matrix {
        Matrix {
            a: matrix.scale_x,
            b: matrix.rotate_skew_1,
            c: matrix.rotate_skew_0,
            d: matrix.scale_y,
            tx: matrix.translate_x,
            ty: matrix.translate_y,
        }
    }
}

impl std::ops::Mul for Matrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: self.a * rhs.tx + self.c * rhs.ty + self.tx,
            ty: self.b * rhs.tx + self.d * rhs.ty + self.ty,
        }
    }
}

impl std::default::Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            a: 1.0,
            c: 0.0,
            tx: 0.0,
            b: 0.0,
            d: 1.0,
            ty: 0.0,
        }
    }
}

impl std::ops::MulAssign for Matrix {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: self.a * rhs.tx + self.c * rhs.ty + self.tx,
            ty: self.b * rhs.tx + self.d * rhs.ty + self.ty,
        }
    }
}

pub struct MatrixStack(Vec<Matrix>);

impl MatrixStack {
    pub fn new() -> MatrixStack {
        MatrixStack(vec![Matrix::default()])
    }

    pub fn push(&mut self, matrix: &Matrix) {
        let new_matrix = *matrix * *self.matrix();
        self.0.push(new_matrix);
    }

    pub fn pop(&mut self) {
        if self.0.len() <= 1 {
            panic!("Matrix stack underflow");
        }
        self.0.pop();
    }

    pub fn matrix(&self) -> &Matrix {
        &self.0[self.0.len() - 1]
    }
}
