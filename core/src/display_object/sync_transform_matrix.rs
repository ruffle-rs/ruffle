// Synchronization/computation between matrix and transform (i.e. scale/rotation/skew).

use ruffle_render::matrix::Matrix;

use crate::types::{Degrees, Percent};

pub(crate) fn props_from_matrix(matrix: Matrix) -> (Degrees, Percent, Percent, f64) {
    let Matrix { a, b, c, d, .. } = matrix;
    let a = f64::from(a);
    let b = f64::from(b);
    let c = f64::from(c);
    let d = f64::from(d);

    // If this object's transform matrix is:
    // [[a c tx]
    //  [b d ty]]
    // After transformation, the X-axis and Y-axis will turn into the column vectors x' = <a, b> and y' = <c, d>.
    // We derive the scale, rotation, and skew values from these transformed axes.
    // The skew value is not exposed by ActionScript, but is remembered internally.
    // xscale = len(x')
    // yscale = len(y')
    // rotation = atan2(b, a)  (the rotation of x' from the normal x-axis).
    // skew = atan2(-c, d) - atan2(b, a)  (the signed difference between y' and x' rotation)

    // This can produce some surprising results due to the overlap between flipping/rotation/skewing.
    // For example, in Flash, using Modify->Transform->Flip Horizontal and then tracing _xscale, _yscale, and _rotation
    // will output 100, 100, and 180. (a horizontal flip could also be a 180 degree skew followed by 180 degree rotation!)
    let rotation_x = f64::atan2(b, a);
    let rotation_y = f64::atan2(-c, d);
    let scale_x = f64::sqrt(a * a + b * b);
    let scale_y = f64::sqrt(c * c + d * d);
    let skew = rotation_y - rotation_x;

    let rotation = Degrees::from_radians(rotation_x);
    let scale_x = Percent::from_unit(scale_x);
    let scale_y = Percent::from_unit(scale_y);

    (rotation, scale_x, scale_y, skew)
}

pub(crate) fn matrix_from_props(
    degrees: Degrees,
    scale_x: Percent,
    scale_y: Percent,
    skew: f64,
) -> Matrix {
    let cos_x = f64::cos(degrees.into_radians());
    let sin_x = f64::sin(degrees.into_radians());
    let cos_y = f64::cos(degrees.into_radians() + skew);
    let sin_y = f64::sin(degrees.into_radians() + skew);
    let scale_x = scale_x.unit();
    let scale_y = scale_y.unit();

    Matrix {
        a: (scale_x * cos_x) as f32,
        b: (scale_x * sin_x) as f32,
        c: (scale_y * -sin_y) as f32,
        d: (scale_y * cos_y) as f32,
        ..Default::default()
    }
}
