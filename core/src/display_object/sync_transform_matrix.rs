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
    // Note - in order to match Flash's behavior, the 'scale_x'/`scale_y` field is set to NaN
    // (which gets reported back to ActionScript), but we treat it as 0 for the purposes of
    // updating the matrix
    let scale_x = nan_to_zero(scale_x.unit());
    let scale_y = nan_to_zero(scale_y.unit());

    // Similarly, a rotation of `NaN` can be reported to ActionScript, but we
    // treat it as 0.0 when calculating the matrix
    let degrees = nan_to_zero(degrees.into_radians());

    let (sin_x, cos_x) = degrees.sin_cos();
    let (sin_y, cos_y) = (degrees + skew).sin_cos();

    Matrix {
        a: (scale_x * cos_x) as f32,
        b: (scale_x * sin_x) as f32,
        c: (scale_y * -sin_y) as f32,
        d: (scale_y * cos_y) as f32,
        ..Default::default()
    }
}

pub(crate) fn to_skip_update(value: f64, _matrix: &Matrix) -> bool {
    // FIXME - this isn't quite correct. In Flash player,
    // trying to set rotation to NaN does nothing if the current
    // matrix 'b' and 'd' terms are both zero. However, if one
    // of those terms is non-zero, then the entire matrix gets
    // modified in a way that depends on its starting values.
    // I haven't been able to figure out how to reproduce those
    // values, so for now, we never modify the matrix if the
    // rotation is NaN. Hopefully, there are no SWFs depending
    // on the weird behavior when b or d is non-zero.
    value.is_nan()
}

#[inline]
fn nan_to_zero(val: f64) -> f64 {
    if val.is_nan() { 0.0 } else { val }
}
