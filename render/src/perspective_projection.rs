use std::f64::consts::PI;

use crate::matrix3d::Matrix3D;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveProjection {
    /// Unit: degree. Must be greater than 0 and less than 180.
    pub field_of_view: f64,

    /// The center of the projection in (x, y).
    pub center: (f64, f64),
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            field_of_view: 55.0,
            center: (250.0, 250.0),
        }
    }
}

impl PerspectiveProjection {
    const DEG2RAD: f64 = PI / 180.0;

    pub fn from_focal_length(focal_length: f64, width: f64) -> Self {
        Self {
            field_of_view: f64::atan((width / 2.0) / focal_length) / Self::DEG2RAD * 2.0,
            ..Default::default()
        }
    }

    pub fn focal_length(&self, width: f32) -> f32 {
        let rad = self.field_of_view * Self::DEG2RAD;
        (width / 2.0) * f64::tan((PI - rad) / 2.0) as f32
    }

    pub fn to_matrix3d(&self, width: f32) -> Matrix3D {
        let focal_length = self.focal_length(width) as f64;

        Matrix3D {
            raw_data: [
                //
                focal_length,
                0.0,
                0.0,
                0.0,
                //
                0.0,
                focal_length,
                0.0,
                0.0,
                //
                0.0,
                0.0,
                1.0,
                1.0,
                //
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        }
    }
}
