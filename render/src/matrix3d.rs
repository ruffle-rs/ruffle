use crate::matrix::Matrix;
use swf::Twips;

/// The transformation matrix for 3D used by Flash display objects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix3D {
    /// 4x4 matrix elements.
    pub raw_data: [f64; 16],
}

impl Matrix3D {
    pub fn from_matrix(matrix: Matrix) -> Self {
        Self {
            raw_data: [
                // 1st column
                matrix.a.into(),
                matrix.b.into(),
                0.0,
                0.0,
                // 2nd column
                matrix.c.into(),
                matrix.d.into(),
                0.0,
                0.0,
                // 3rd column
                0.0,
                0.0,
                1.0,
                0.0,
                // 4th column
                matrix.tx.to_pixels(),
                matrix.ty.to_pixels(),
                0.0,
                1.0,
            ],
        }
    }

    pub fn to_matrix(self) -> Matrix {
        Matrix {
            a: self.raw_data[0] as f32,
            b: self.raw_data[1] as f32,
            c: self.raw_data[4] as f32,
            d: self.raw_data[5] as f32,
            tx: Twips::from_pixels(self.raw_data[12]),
            ty: Twips::from_pixels(self.raw_data[13]),
        }
    }
}

impl Matrix3D {
    pub const ZERO: Self = Self {
        raw_data: [0.0; 16],
    };
    pub const IDENTITY: Self = Self {
        raw_data: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn tz(&self) -> f64 {
        self.raw_data[14]
    }
    pub fn set_tz(&mut self, tz: f64) {
        self.raw_data[14] = tz;
    }
}

impl std::ops::Mul for Matrix3D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = Matrix3D::ZERO;
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    res.raw_data[i + 4 * j] += self.raw_data[i + 4 * k] * rhs.raw_data[k + 4 * j];
                }
            }
        }
        res
    }
}
