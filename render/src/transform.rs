use crate::matrix::Matrix;
use crate::matrix3d::Matrix3D;
use crate::perspective_projection::PerspectiveProjection;
use swf::{ColorTransform, Twips};

/// Represents the transform for a DisplayObject.
/// This includes both the transformation matrix and the color transform.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub matrix: Matrix,
    pub tz: f64,
    /// Flash `DisplayObject.rotationX`, in degrees.
    pub rotation_x: f64,
    /// Flash `DisplayObject.rotationY`, in degrees.
    pub rotation_y: f64,
    /// Concatenated 4x4 when an ancestor used display-list 3D.
    /// When set, `to_matrix3d` returns this directly.
    pub world: Option<Matrix3D>,
    pub color_transform: ColorTransform,
    pub perspective_projection: Option<PerspectiveProjection>,
    /// Extra scale applied to world-space z at draw time.
    ///
    /// The 2D viewport matrix scales x/y into framebuffer pixels (HiDPI,
    /// ShowAll, ExactFit), but `DisplayObject.z` stays in stage pixels.
    /// Perspective then treats xyz as the same unit, which inflates the
    /// effective focal length and makes the camera look zoomed out.
    /// This scale (typically the viewport matrix's x-axis length) puts z
    /// in the same space as x/y before the projection.
    pub view_z_scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            matrix: Matrix::IDENTITY,
            tz: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            world: None,
            color_transform: ColorTransform::default(),
            perspective_projection: None,
            view_z_scale: 1.0,
        }
    }
}

impl Transform {
    pub fn has_3d_rotation(&self) -> bool {
        self.world.is_some() || self.rotation_x != 0.0 || self.rotation_y != 0.0
    }

    /// True when this concatenated transform uses display-list 3D.
    /// 2D world AABBs are not a valid cull test in that case.
    pub fn uses_3d(&self) -> bool {
        self.has_3d_rotation() || self.tz != 0.0
    }

    pub fn to_matrix3d(&self) -> Matrix3D {
        if let Some(world) = self.world {
            return world;
        }
        compose_display_list_3d(self.matrix, self.tz, self.rotation_x, self.rotation_y)
    }

    pub fn world_matrix_cols(&self) -> [[f32; 4]; 4] {
        let mut m = self.to_matrix3d();
        let s = self.view_z_scale;
        if s != 1.0 {
            // Scale the world-space z row of each basis vector and translation.
            m.raw_data[2] *= s;
            m.raw_data[6] *= s;
            m.raw_data[10] *= s;
            m.raw_data[14] *= s;
        }
        m.as_cols()
    }

    /// Append a local 2D matrix (bitmap size, pixel snap extras) before world.
    pub fn with_local_matrix(&self, extra: Matrix) -> Self {
        let mut next = self.clone();
        if let Some(world) = next.world {
            next.world = Some(world.multiply(&Matrix3D::from_matrix(extra)));
        } else {
            next.matrix = next.matrix * extra;
        }
        next
    }
}

/// Local display-list 3D: `T(tx, ty, tz) * Rz * Ry * Rx * S`.
///
/// This is Flash `Matrix3D.recompose` with `Orientation3D.EULER_ANGLES`,
/// which is what `DisplayObject` uses when you set `x`/`y`/`z`,
/// `rotation`/`rotationX`/`rotationY`, and `scaleX`/`scaleY`.
///
/// Vertices sit on the object's z=0 plane. Scale/skew run first, then pitch
/// (`rotationX`), yaw (`rotationY`), then the 2D `rotation` (Z), then
/// translation. Hit tests stay on the 2D `matrix`.
pub fn compose_display_list_3d(
    matrix: Matrix,
    tz: f64,
    rotation_x_deg: f64,
    rotation_y_deg: f64,
) -> Matrix3D {
    // Keep the 2D+z path bit-identical when there is no 3D rotation.
    if rotation_x_deg == 0.0 && rotation_y_deg == 0.0 {
        let mut m = Matrix3D::from_matrix(matrix);
        m.set_tz(tz as f32);
        return m;
    }

    let translation = Matrix3D::translate(
        matrix.tx.to_pixels() as f32,
        matrix.ty.to_pixels() as f32,
        tz as f32,
    );
    let rot_y = Matrix3D::rotate_y((rotation_y_deg as f32).to_radians());
    let rot_x = Matrix3D::rotate_x((rotation_x_deg as f32).to_radians());

    // Split the 2D linear part into Rz * S so 2D `rotation` is applied
    // after pitch/yaw, matching Flash Euler order.
    let (scale_linear, rot_z) = split_2d_rotation(matrix);
    translation.multiply(&rot_z.multiply(&rot_y.multiply(&rot_x.multiply(&scale_linear))))
}

/// Factor `Linear2D = Rz(atan2(b, a)) * S` and return `(S, Rz)`.
fn split_2d_rotation(matrix: Matrix) -> (Matrix3D, Matrix3D) {
    let angle = matrix.b.atan2(matrix.a);
    let (sin, cos) = angle.sin_cos();
    // Rz^{-1} * |a c|
    //            |b d|
    let a = cos * matrix.a + sin * matrix.b;
    let b = -sin * matrix.a + cos * matrix.b;
    let c = cos * matrix.c + sin * matrix.d;
    let d = -sin * matrix.c + cos * matrix.d;
    let scale_linear = Matrix3D::from_matrix(Matrix {
        a,
        b,
        c,
        d,
        tx: Twips::ZERO,
        ty: Twips::ZERO,
    });
    (scale_linear, Matrix3D::rotate_z(angle))
}

pub struct TransformStack(Vec<Transform>);

impl TransformStack {
    pub fn new() -> Self {
        Self(vec![Transform::default()])
    }

    pub fn push(&mut self, transform: &Transform) {
        let cur_transform = self.transform();
        let matrix = cur_transform.matrix * transform.matrix;
        let color_transform = cur_transform.color_transform * transform.color_transform;
        let tz = cur_transform.tz + transform.tz;
        let world = if cur_transform.has_3d_rotation() || transform.has_3d_rotation() {
            Some(
                cur_transform
                    .to_matrix3d()
                    .multiply(&transform.to_matrix3d()),
            )
        } else {
            None
        };
        self.0.push(Transform {
            matrix,
            color_transform,
            // TODO: Merge perspective_projections from cur_transform and transform properly
            perspective_projection: Default::default(),
            tz,
            rotation_x: 0.0,
            rotation_y: 0.0,
            world,
            view_z_scale: cur_transform.view_z_scale * transform.view_z_scale,
        });
    }

    pub fn pop(&mut self) {
        assert!(self.0.len() > 1, "Transform stack underflow");
        self.0.pop();
    }

    pub fn transform(&self) -> Transform {
        self.0[self.0.len() - 1].clone()
    }
}

impl Default for TransformStack {
    fn default() -> Self {
        TransformStack::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::Matrix;
    use swf::Twips;

    #[test]
    fn compose_without_rotation_matches_2d_matrix() {
        let matrix = Matrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 3.0,
            tx: Twips::from_pixels(10.0),
            ty: Twips::from_pixels(20.0),
        };
        let composed = compose_display_list_3d(matrix, 5.0, 0.0, 0.0);
        let mut expected = Matrix3D::from_matrix(matrix);
        expected.set_tz(5.0);
        assert_eq!(composed, expected);
    }

    #[test]
    fn compose_rotation_x_tilts_y_axis() {
        let composed = compose_display_list_3d(Matrix::IDENTITY, 0.0, 90.0, 0.0);
        // Rx(90°) sends +Y toward +Z in column-major Flash Matrix3D.
        let (x, y, z) = composed.transform_xyz(0.0, 1.0, 0.0);
        assert!((x - 0.0).abs() < 1e-5);
        assert!((y - 0.0).abs() < 1e-5);
        assert!((z - 1.0).abs() < 1e-5);
    }

    #[test]
    fn compose_matches_flash_euler_recompose() {
        // Flash DisplayObject matrix3D is T * Rz * Ry * Rx * S.
        // A 2D rotation of 90° plus rotationX of 90° must apply pitch first
        // (Y → Z) then yaw around Z, not the reverse.
        let matrix = Matrix {
            a: 0.0,
            b: 1.0,
            c: -1.0,
            d: 0.0,
            tx: Twips::from_pixels(10.0),
            ty: Twips::from_pixels(20.0),
        };
        let composed = compose_display_list_3d(matrix, 5.0, 90.0, 0.0);

        let translation = Matrix3D::translate(10.0, 20.0, 5.0);
        let expected = translation.multiply(
            &Matrix3D::rotate_z(90f32.to_radians())
                .multiply(&Matrix3D::rotate_x(90f32.to_radians())),
        );
        for (got, want) in composed.raw_data.iter().zip(expected.raw_data.iter()) {
            assert!((got - want).abs() < 1e-5, "{composed:?} vs {expected:?}");
        }

        // +X is rotated by Rz(90°) onto +Y: (1,0,0) → (0,1,0)
        assert!((composed.raw_data[0] - 0.0).abs() < 1e-5);
        assert!((composed.raw_data[1] - 1.0).abs() < 1e-5);
        assert!((composed.raw_data[2] - 0.0).abs() < 1e-5);
        // +Y is pitched onto +Z, then Rz(90°) leaves Z as Z: (0,1,0) → (0,0,1)
        assert!((composed.raw_data[4] - 0.0).abs() < 1e-5);
        assert!((composed.raw_data[5] - 0.0).abs() < 1e-5);
        assert!((composed.raw_data[6] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn view_z_scale_matches_xy_before_projection() {
        let transform = Transform {
            matrix: Matrix {
                a: 2.0,
                d: 2.0,
                tx: Twips::from_pixels(10.0),
                ty: Twips::from_pixels(20.0),
                ..Default::default()
            },
            tz: 50.0,
            view_z_scale: 2.0,
            ..Default::default()
        };
        let cols = transform.world_matrix_cols();
        // Translation xyz should all be in framebuffer pixels.
        assert!((cols[3][0] - 10.0).abs() < 1e-5);
        assert!((cols[3][1] - 20.0).abs() < 1e-5);
        assert!((cols[3][2] - 100.0).abs() < 1e-5);
    }
}
