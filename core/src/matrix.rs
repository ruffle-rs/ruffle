use swf::Twips;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Matrix {
    pub fn invert(&mut self) {
        let det = self.a * self.d - self.b * self.c;
        let a = self.d / det;
        let b = self.b / -det;
        let c = self.c / -det;
        let d = self.a / det;
        let tx = (self.d * self.tx - self.c * self.ty) / -det;
        let ty = (self.b * self.tx - self.a * self.ty) / det;
        *self = Matrix { a, b, c, d, tx, ty };
    }
}

impl From<swf::Matrix> for Matrix {
    fn from(matrix: swf::Matrix) -> Matrix {
        Matrix {
            a: matrix.scale_x,
            b: matrix.rotate_skew_0,
            c: matrix.rotate_skew_1,
            d: matrix.scale_y,
            tx: matrix.translate_x.get() as f32,
            ty: matrix.translate_y.get() as f32,
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

impl std::ops::Mul<(Twips, Twips)> for Matrix {
    type Output = (Twips, Twips);
    fn mul(self, (x, y): (Twips, Twips)) -> (Twips, Twips) {
        let (x, y) = (x.get() as f32, y.get() as f32);
        let out_x = self.a * x + self.c * y + self.tx;
        let out_y = self.b * x + self.d * y + self.ty;
        (Twips::new(out_x as i32), Twips::new(out_y as i32))
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
