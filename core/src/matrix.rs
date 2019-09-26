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

#[cfg(test)]
mod tests {
    use super::*;

    // Identity matrix inverted shouldn't change
    #[test]
    fn inverted_identity_matrix() {
        let mut mat : Matrix = Default::default();
        mat.invert();

        let inverted : Matrix = Default::default();
        assert_eq!(mat, inverted);
    }

    // Test a matrix that can't be inverted
    #[test]
    fn inverted_all_zero_matrix() {
        let mut mat = Matrix {
            a: 0.0,
            c: 0.0,
            tx: 0.0,
            b: 0.0,
            d: 0.0,
            ty: 0.0,
        };
        mat.invert();

        assert!(mat.a.is_nan());
        assert!(mat.b.is_nan());
        assert!(mat.c.is_nan());
        assert!(mat.d.is_nan());
        assert!(mat.tx.is_nan());
        assert!(mat.ty.is_nan());
    }

    // Test that a scaled up identity matrix should make the diagonals the reciprocals
    #[test]
    fn inverted_scaled_identity_matrix() {
        let mut mat = Matrix {
            a: 3.0,
            c: 0.0,
            tx: 0.0,
            b: 0.0,
            d: 3.0,
            ty: 0.0,
        };
        mat.invert();

        let inverted = Matrix {
            a: 1.0 / 3.0,
            c: 0.0,
            tx: 0.0,
            b: 0.0,
            d: 1.0 / 3.0,
            ty: 0.0,
        };
        assert_eq!(mat, inverted);
    }

    // Test inverting a matrix that has values > 1 for all configurable values
    #[test]
    fn inverted_full_matrix() {
        let mut mat = Matrix {
            a: 1.0,
            c: 4.0,
            tx: 7.0,
            b: 2.0,
            d: 5.0,
            ty: 2.0,
        };
        mat.invert();

        let inverted = Matrix {
            a: -5.0 / 3.0,
            c: 4.0 / 3.0,
            tx: 9.0,
            b: 2.0 / 3.0,
            d: -1.0 / 3.0,
            ty: -4.0,
        };
        assert_eq!(mat, inverted);
    }

    // Test inverting a matrix with negative values
    #[test]
    fn inverted_negative_matrix() {
        let mut mat = Matrix {
            a: -1.0,
            c: -4.0,
            tx: -7.0,
            b: -2.0,
            d: -5.0,
            ty: -2.0,
        };
        mat.invert();

        let inverted = Matrix {
            a: 5.0 / 3.0,
            c: -4.0 / 3.0,
            tx: 9.0,
            b: -2.0 / 3.0,
            d: 1.0 / 3.0,
            ty: -4.0,
        };
        
        assert_eq!(mat, inverted);
    }

    // Test that a matrix multipled by identity is unchanged
    #[test]
    fn multiply_by_identity_matrix() {
        let mat = Matrix {
            a: 1.0,
            c: 4.0,
            tx: 7.0,
            b: 2.0,
            d: 5.0,
            ty: 2.0,
        };

        let identity : Matrix = Default::default();

        assert_eq!(mat * identity, mat);
    }

    // Test basic matrix multiplication
    #[test]
    fn multiply_matrix() {
        let mat = Matrix {
            a: 1.0,
            c: 4.0,
            tx: 7.0,
            b: 2.0,
            d: 5.0,
            ty: 2.0,
        };

        let result = Matrix {
            a: 9.0,
            c: 24.0,
            tx: 22.0,
            b: 12.0,
            d: 33.0,
            ty: 26.0,
        };

        assert_eq!(mat * mat, result);
    }

    // The identity matrix effecitvely represents no transformation; i.e. all coordinates stay in the same place
    #[test]
    fn identy_matrix_doesnt_change_twips() {
        let identity : Matrix = Default::default();
        let pos1 = (Twips::new(0), Twips::new(0));
        let pos2 = (Twips::new(10), Twips::new(10));

        assert_eq!(identity * pos1, pos1);
        assert_eq!(identity * pos2, pos2);
    }

    // tx and ty should control translation of twips
    #[test]
    fn simple_twips_transform() {
        let mat = Matrix {
            a: 1.0,
            c: 0.0,
            tx: 5.0,
            b: 0.0,
            d: 1.0,
            ty: -5.0,
        };


        let pos1 = (Twips::new(0), Twips::new(0));
        let pos2 = (Twips::new(10), Twips::new(10));

        assert_eq!(mat * pos1, (Twips::new(5), Twips::new(-5)));
        assert_eq!(mat * pos2, (Twips::new(15), Twips::new(5)));
    }
    
}
