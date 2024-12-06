use crate::matrix::Matrix;
use swf::Twips;

/// The transformation matrix for 3D used by Flash display objects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix3D {
    /// 4x4 matrix elements.
    pub raw_data: [f64; 16],
}

impl From<Matrix> for Matrix3D {
    fn from(matrix: Matrix) -> Self {
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
}
impl From<Matrix3D> for Matrix {
    fn from(matrix: Matrix3D) -> Self {
        Self {
            a: matrix.raw_data[0] as f32,
            b: matrix.raw_data[1] as f32,
            c: matrix.raw_data[4] as f32,
            d: matrix.raw_data[5] as f32,
            tx: Twips::from_pixels(matrix.raw_data[12]),
            ty: Twips::from_pixels(matrix.raw_data[13]),
        }
    }
}
