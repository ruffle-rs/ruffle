use crate::matrix::Matrix;
use swf::Twips;

/// The transformation matrix for 3D used by Flash display objects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix3D {
    /// 4x4 matrix elements.
    pub raw_data: [f64; 16],
}

impl Matrix3D {
    pub const IDENTITY: Self = Self {
        raw_data: [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ],
    };

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

    pub fn transpose_in_place(&mut self) {
        self.raw_data.swap(1, 4);
        self.raw_data.swap(2, 8);
        self.raw_data.swap(3, 12);
        self.raw_data.swap(6, 9);
        self.raw_data.swap(7, 13);
        self.raw_data.swap(11, 14);
    }

    pub fn determinant(&self) -> f64 {
        let m = &self.raw_data;
        (m[0] * m[5] - m[4] * m[1]) * (m[10] * m[15] - m[14] * m[11])
            - (m[0] * m[9] - m[8] * m[1]) * (m[6] * m[15] - m[14] * m[7])
            + (m[0] * m[13] - m[12] * m[1]) * (m[6] * m[11] - m[10] * m[7])
            + (m[4] * m[9] - m[8] * m[5]) * (m[2] * m[15] - m[14] * m[3])
            - (m[4] * m[13] - m[12] * m[5]) * (m[2] * m[11] - m[10] * m[3])
            + (m[8] * m[13] - m[12] * m[9]) * (m[2] * m[7] - m[6] * m[3])
    }

    pub fn multiply(&self, rhs: &Self) -> Self {
        let lhs = &self.raw_data;
        let rhs = &rhs.raw_data;
        let mut result = [0.0; 16];

        for column in 0..4 {
            for row in 0..4 {
                result[column * 4 + row] = lhs[row] * rhs[column * 4]
                    + lhs[4 + row] * rhs[column * 4 + 1]
                    + lhs[8 + row] * rhs[column * 4 + 2]
                    + lhs[12 + row] * rhs[column * 4 + 3];
            }
        }

        Self { raw_data: result }
    }
}
