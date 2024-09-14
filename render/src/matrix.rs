use swf::{Fixed16, Point, PointDelta, Rectangle, Twips};

/// TODO: Consider using portable SIMD when it's stable (https://doc.rust-lang.org/std/simd/index.html).

/// The transformation matrix used by Flash display objects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix {
    /// Serialized as `scale_x` in SWF files
    pub a: f32,

    /// Serialized as `rotate_skew_0` in SWF files
    pub b: f32,

    /// Serialized as `rotate_skew_1` in SWF files
    pub c: f32,

    /// Serialized as `scale_y` in SWF files
    pub d: f32,

    /// Serialized as `transform_x` in SWF files
    pub tx: Twips,

    /// Serialized as `transform_y` in SWF files
    pub ty: Twips,
}

impl Matrix {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 1.0,
        ty: Twips::ZERO,
    };

    pub const ZERO: Self = Self {
        a: 0.0,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 0.0,
        ty: Twips::ZERO,
    };

    pub const TWIPS_TO_PIXELS: Self = Self {
        a: 1.0 / Twips::TWIPS_PER_PIXEL as f32,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 1.0 / Twips::TWIPS_PER_PIXEL as f32,
        ty: Twips::ZERO,
    };

    pub const PIXELS_TO_TWIPS: Self = Self {
        a: Twips::TWIPS_PER_PIXEL as f32,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: Twips::TWIPS_PER_PIXEL as f32,
        ty: Twips::ZERO,
    };

    pub const fn scale(scale_x: f32, scale_y: f32) -> Self {
        Self {
            a: scale_x,
            c: 0.0,
            tx: Twips::ZERO,
            b: 0.0,
            d: scale_y,
            ty: Twips::ZERO,
        }
    }

    pub fn rotate(angle: f32) -> Self {
        Self {
            a: angle.cos(),
            c: -angle.sin(),
            tx: Twips::ZERO,
            b: angle.sin(),
            d: angle.cos(),
            ty: Twips::ZERO,
        }
    }

    pub fn translate(x: Twips, y: Twips) -> Self {
        Self {
            a: 1.0,
            c: 0.0,
            tx: x,
            b: 0.0,
            d: 1.0,
            ty: y,
        }
    }

    pub fn create_box(scale_x: f32, scale_y: f32, translate_x: Twips, translate_y: Twips) -> Self {
        Self {
            a: scale_x,
            c: 0.0,
            tx: translate_x,
            b: 0.0,
            d: scale_y,
            ty: translate_y,
        }
    }

    pub fn create_box_with_rotation(
        scale_x: f32,
        scale_y: f32,
        rotation: f32,
        translate_x: Twips,
        translate_y: Twips,
    ) -> Self {
        Self {
            a: rotation.cos() * scale_x,
            c: -rotation.sin() * scale_x,
            tx: translate_x,
            b: rotation.sin() * scale_y,
            d: rotation.cos() * scale_y,
            ty: translate_y,
        }
    }

    pub fn create_gradient_box(
        width: f32,
        height: f32,
        rotation: f32,
        translate_x: Twips,
        translate_y: Twips,
    ) -> Self {
        Self::create_box_with_rotation(
            width / 1638.4,
            height / 1638.4,
            rotation,
            translate_x + Twips::from_pixels((width / 2.0) as f64),
            translate_y + Twips::from_pixels((height / 2.0) as f64),
        )
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        let (tx, ty) = (self.tx.get() as f32, self.ty.get() as f32);
        let det = self.determinant();
        if det.abs() > f32::EPSILON {
            let a = self.d / det;
            let b = self.b / -det;
            let c = self.c / -det;
            let d = self.a / det;
            let (out_tx, out_ty) = (
                round_to_i32((self.d * tx - self.c * ty) / -det),
                round_to_i32((self.b * tx - self.a * ty) / det),
            );
            Some(Matrix {
                a,
                b,
                c,
                d,
                tx: Twips::new(out_tx),
                ty: Twips::new(out_ty),
            })
        } else {
            None
        }
    }
}

impl std::ops::Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let (rhs_tx, rhs_ty) = (rhs.tx.get() as f32, rhs.ty.get() as f32);
        let (out_tx, out_ty) = (
            round_to_i32(self.a * rhs_tx + self.c * rhs_ty).wrapping_add(self.tx.get()),
            round_to_i32(self.b * rhs_tx + self.d * rhs_ty).wrapping_add(self.ty.get()),
        );
        Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: Twips::new(out_tx),
            ty: Twips::new(out_ty),
        }
    }
}

impl std::ops::Mul<Point<Twips>> for Matrix {
    type Output = Point<Twips>;

    fn mul(self, point: Point<Twips>) -> Point<Twips> {
        let x = point.x.get() as f32;
        let y = point.y.get() as f32;
        let out_x = Twips::new(round_to_i32(self.a * x + self.c * y).wrapping_add(self.tx.get()));
        let out_y = Twips::new(round_to_i32(self.b * x + self.d * y).wrapping_add(self.ty.get()));
        Point::new(out_x, out_y)
    }
}

impl std::ops::Mul<PointDelta<Twips>> for Matrix {
    type Output = PointDelta<Twips>;

    fn mul(self, delta: PointDelta<Twips>) -> PointDelta<Twips> {
        let dx = delta.dx.get() as f32;
        let dy = delta.dy.get() as f32;
        let out_dx = Twips::new(round_to_i32(self.a * dx + self.c * dy));
        let out_dy = Twips::new(round_to_i32(self.b * dx + self.d * dy));
        PointDelta::new(out_dx, out_dy)
    }
}

impl std::ops::Mul<Rectangle<Twips>> for Matrix {
    type Output = Rectangle<Twips>;

    fn mul(self, rhs: Rectangle<Twips>) -> Self::Output {
        if !rhs.is_valid() {
            return Default::default();
        }

        let p0 = self * Point::new(rhs.x_min, rhs.y_min);
        let p1 = self * Point::new(rhs.x_min, rhs.y_max);
        let p2 = self * Point::new(rhs.x_max, rhs.y_min);
        let p3 = self * Point::new(rhs.x_max, rhs.y_max);
        Rectangle {
            x_min: p0.x.min(p1.x).min(p2.x).min(p3.x),
            x_max: p0.x.max(p1.x).max(p2.x).max(p3.x),
            y_min: p0.y.min(p1.y).min(p2.y).min(p3.y),
            y_max: p0.y.max(p1.y).max(p2.y).max(p3.y),
        }
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix::IDENTITY
    }
}

impl std::ops::MulAssign for Matrix {
    fn mul_assign(&mut self, rhs: Self) {
        let (rhs_tx, rhs_ty) = (rhs.tx.get() as f32, rhs.ty.get() as f32);
        let (out_tx, out_ty) = (
            round_to_i32(self.a * rhs_tx + self.c * rhs_ty) + self.tx.get(),
            round_to_i32(self.b * rhs_tx + self.d * rhs_ty) + self.ty.get(),
        );
        *self = Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: Twips::new(out_tx),
            ty: Twips::new(out_ty),
        }
    }
}

impl From<swf::Matrix> for Matrix {
    fn from(matrix: swf::Matrix) -> Self {
        Self {
            a: matrix.a.to_f32(),
            b: matrix.b.to_f32(),
            c: matrix.c.to_f32(),
            d: matrix.d.to_f32(),
            tx: matrix.tx,
            ty: matrix.ty,
        }
    }
}

impl From<Matrix> for swf::Matrix {
    fn from(matrix: Matrix) -> Self {
        Self {
            a: Fixed16::from_f32(matrix.a),
            b: Fixed16::from_f32(matrix.b),
            c: Fixed16::from_f32(matrix.c),
            d: Fixed16::from_f32(matrix.d),
            tx: matrix.tx,
            ty: matrix.ty,
        }
    }
}

/// Implements the IEEE-754 "Round to nearest, ties to even" rounding rule.
/// (e.g., both 1.5 and 2.5 will round to 2).
/// This is the rounding method used by Flash for the above transforms.
/// This also clamps out-of-range values and NaN to `i32::MIN`.
fn round_to_i32(f: f32) -> i32 {
    if f.is_finite() {
        if f < 2_147_483_648.0_f32 {
            f.round_ties_even() as i32
        } else {
            // Out-of-range clamps to MIN.
            i32::MIN
        }
    } else {
        // NaN/Infinity goes to 0.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_ulps_eq, AbsDiffEq, UlpsEq};

    macro_rules! test_inverse {
        ($test: ident, $($args: expr),* $(,)?) => {
            #[test]
            fn $test() {
                $(
                    let (input, output) = $args;
                    match (input.inverse(), output) {
                        (Some(result), Some(output)) => assert_ulps_eq!(result, output),
                        (None, None) => (),
                        (result, output) => panic!("Matrix::inverse: Got {:?}, expected {:?}", result, output),
                    }
                )*
            }
        };
    }

    macro_rules! test_multiply {
        ($test: ident, $($args: expr),* $(,)?) => {
            #[test]
            fn $test() {
                $(
                    let (input1, input2, output) = $args;
                    assert_ulps_eq!(input1 * input2, output);
                )*
            }
        };
    }

    macro_rules! test_multiply_twips {
        ($test: ident, $($args: expr),* $(,)?) => {
            #[test]
            fn $test() {
                $(
                    let (input1, input2, output) = $args;
                    assert_eq!(input1 * input2, output);
                )*
            }
        };
    }

    impl AbsDiffEq for Matrix {
        type Epsilon = (<f32 as AbsDiffEq>::Epsilon, <i32 as AbsDiffEq>::Epsilon);
        fn default_epsilon() -> Self::Epsilon {
            (f32::default_epsilon(), i32::default_epsilon())
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.a.abs_diff_eq(&other.a, epsilon.0)
                && self.b.abs_diff_eq(&other.b, epsilon.0)
                && self.c.abs_diff_eq(&other.c, epsilon.0)
                && self.d.abs_diff_eq(&other.d, epsilon.0)
                && self.tx.get().abs_diff_eq(&other.tx.get(), epsilon.1)
                && self.ty.get().abs_diff_eq(&other.ty.get(), epsilon.1)
        }
    }

    impl UlpsEq for Matrix {
        fn default_max_ulps() -> u32 {
            f32::default_max_ulps()
        }

        fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
            self.a.ulps_eq(&other.a, epsilon.0, max_ulps)
                && self.b.ulps_eq(&other.b, epsilon.0, max_ulps)
                && self.c.ulps_eq(&other.c, epsilon.0, max_ulps)
                && self.d.ulps_eq(&other.d, epsilon.0, max_ulps)
                && self.tx == other.tx
                && self.ty == other.ty
        }
    }

    // Identity matrix inverted should be unchanged.
    test_inverse!(
        inverse_identity_matrix,
        (Matrix::default(), Some(Matrix::default())),
    );

    // Standard test cases; there's nothing special about these matrices.
    test_inverse!(
        inverse_matrix,
        (
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: Twips::from_pixels(7.0),
                b: 2.0,
                d: 5.0,
                ty: Twips::from_pixels(2.0),
            },
            Some(Matrix {
                a: -1.666_666_6,
                c: 1.333_333_3,
                tx: Twips::from_pixels(9.0),
                b: 0.666_666_6,
                d: -0.333_333_3,
                ty: Twips::from_pixels(-4.0),
            }),
        ),
        (
            Matrix {
                a: -1.0,
                c: -4.0,
                tx: Twips::from_pixels(-7.0),
                b: -2.0,
                d: -5.0,
                ty: Twips::from_pixels(-2.0),
            },
            Some(Matrix {
                a: 1.666_666_6,
                c: -1.333_333_3,
                tx: Twips::from_pixels(9.0),
                b: -0.666_666_6,
                d: 0.333_333_3,
                ty: Twips::from_pixels(-4.0),
            }),
        ),
        (
            Matrix {
                a: 1.5,
                c: 1.2,
                tx: Twips::from_pixels(1.0),
                b: -2.7,
                d: 3.4,
                ty: Twips::from_pixels(-2.4),
            },
            Some(Matrix {
                a: 0.407_673_9,
                c: -0.143_884_9,
                tx: Twips::from_pixels(-0.752_997_6),
                b: 0.323_741,
                d: 0.179_856_1,
                ty: Twips::from_pixels(0.107_913_67),
            }),
        ),
        (
            Matrix {
                a: -2.0,
                c: 0.0,
                tx: Twips::from_pixels(10.0),
                b: 0.0,
                d: -1.0,
                ty: Twips::from_pixels(5.0),
            },
            Some(Matrix {
                a: -0.5,
                c: 0.0,
                tx: Twips::from_pixels(5.0),
                b: 0.0,
                d: -1.0,
                ty: Twips::from_pixels(5.0),
            }),
        ),
    );

    // Non-invertible matrices
    test_inverse!(
        inverse_uninvertible_matrices,
        (Matrix::ZERO, None),
        (
            Matrix {
                a: 8.0,
                b: 2.0,
                c: 16.0,
                d: 4.0,
                tx: Twips::from_pixels(123.0),
                ty: Twips::from_pixels(234.0),
            },
            None,
        ),
    );

    // Anything multiplied by the identity matrix should be unchanged.
    test_multiply!(
        multiply_identity_matrix,
        (Matrix::default(), Matrix::default(), Matrix::default()),
        (
            Matrix::default(),
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: Twips::from_pixels(7.0),
                b: 2.0,
                d: 5.0,
                ty: Twips::from_pixels(2.0),
            },
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: Twips::from_pixels(7.0),
                b: 2.0,
                d: 5.0,
                ty: Twips::from_pixels(2.0),
            }
        ),
        (
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: Twips::from_pixels(7.0),
                b: 2.0,
                d: 5.0,
                ty: Twips::from_pixels(2.0),
            },
            Matrix::default(),
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: Twips::from_pixels(7.0),
                b: 2.0,
                d: 5.0,
                ty: Twips::from_pixels(2.0),
            },
        ),
    );

    // General test cases for matrix multiplication.
    test_multiply!(
        multiply_matrices,
        (
            Matrix {
                a: 6.0,
                c: 4.0,
                tx: Twips::new(2),
                b: 5.0,
                d: 3.0,
                ty: Twips::new(1),
            },
            Matrix {
                a: 1.0,
                c: 3.0,
                tx: Twips::new(5),
                b: 2.0,
                d: 4.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 14.0,
                c: 34.0,
                tx: Twips::new(56),
                b: 11.0,
                d: 27.0,
                ty: Twips::new(44),
            },
        ),
        (
            Matrix {
                a: 1.0,
                c: 3.0,
                tx: Twips::new(5),
                b: 2.0,
                d: 4.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 6.0,
                c: 4.0,
                tx: Twips::new(2),
                b: 5.0,
                d: 3.0,
                ty: Twips::new(1),
            },
            Matrix {
                a: 21.0,
                c: 13.0,
                tx: Twips::new(10),
                b: 32.0,
                d: 20.0,
                ty: Twips::new(14),
            },
        ),
        (
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: Twips::new(3),
                b: 4.0,
                d: 5.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 6.0,
                c: 5.0,
                tx: Twips::new(4),
                b: 3.0,
                d: 2.0,
                ty: Twips::new(1),
            },
            Matrix {
                a: 12.0,
                c: 9.0,
                tx: Twips::new(9),
                b: 39.0,
                d: 30.0,
                ty: Twips::new(27),
            },
        ),
        (
            Matrix {
                a: 6.0,
                c: 5.0,
                tx: Twips::new(4),
                b: 3.0,
                d: 2.0,
                ty: Twips::new(1),
            },
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: Twips::new(3),
                b: 4.0,
                d: 5.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 26.0,
                c: 37.0,
                tx: Twips::new(52),
                b: 11.0,
                d: 16.0,
                ty: Twips::new(22),
            },
        ),
        (
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: Twips::new(3),
                b: 4.0,
                d: 5.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: Twips::new(3),
                b: 4.0,
                d: 5.0,
                ty: Twips::new(6),
            },
            Matrix {
                a: 9.0,
                c: 12.0,
                tx: Twips::new(18),
                b: 24.0,
                d: 33.0,
                ty: Twips::new(48),
            },
        ),
    );

    // Twips multiplied by the identity/default matrix should be unchanged.
    test_multiply_twips!(
        multiply_twips_identity_matrix,
        (Matrix::default(), Point::ZERO, Point::ZERO),
        (Matrix::default(), PointDelta::ZERO, PointDelta::ZERO),
        (
            Matrix::default(),
            Point::new(Twips::ZERO, Twips::new(10)),
            Point::new(Twips::ZERO, Twips::new(10)),
        ),
        (
            Matrix::default(),
            PointDelta::new(Twips::ZERO, Twips::new(10)),
            PointDelta::new(Twips::ZERO, Twips::new(10)),
        ),
        (
            Matrix::default(),
            Point::new(Twips::new(10), Twips::ZERO),
            Point::new(Twips::new(10), Twips::ZERO),
        ),
        (
            Matrix::default(),
            PointDelta::new(Twips::new(10), Twips::ZERO),
            PointDelta::new(Twips::new(10), Twips::ZERO),
        ),
        (
            Matrix::default(),
            Point::new(Twips::new(-251), Twips::new(152)),
            Point::new(Twips::new(-251), Twips::new(152)),
        ),
        (
            Matrix::default(),
            PointDelta::new(Twips::new(-251), Twips::new(152)),
            PointDelta::new(Twips::new(-251), Twips::new(152)),
        ),
    );

    // Multiply by translate matrices; points should be shifted, point deltas should be unchanged.
    test_multiply_twips!(
        multiply_twips_translate,
        (
            Matrix::translate(Twips::new(10), Twips::new(5)),
            Point::ZERO,
            Point::new(Twips::new(10), Twips::new(5)),
        ),
        (
            Matrix::translate(Twips::new(10), Twips::new(5)),
            PointDelta::ZERO,
            PointDelta::ZERO,
        ),
        (
            Matrix::translate(Twips::new(-200), Twips::ZERO),
            Point::new(Twips::new(50), Twips::new(20)),
            Point::new(Twips::new(-150), Twips::new(20)),
        ),
        (
            Matrix::translate(Twips::new(-200), Twips::ZERO),
            PointDelta::new(Twips::new(50), Twips::new(20)),
            PointDelta::new(Twips::new(50), Twips::new(20)),
        ),
    );

    // Multiply by scalar matrices; values should be scaled up/down.
    test_multiply_twips!(
        multiply_twips_scale,
        (Matrix::scale(3.0, 3.0), Point::ZERO, Point::ZERO),
        (Matrix::scale(3.0, 3.0), PointDelta::ZERO, PointDelta::ZERO),
        (
            Matrix::scale(3.0, 3.0),
            Point::new(Twips::new(10), Twips::new(10)),
            Point::new(Twips::new(30), Twips::new(30)),
        ),
        (
            Matrix::scale(3.0, 3.0),
            PointDelta::new(Twips::new(10), Twips::new(10)),
            PointDelta::new(Twips::new(30), Twips::new(30)),
        ),
        (
            Matrix::scale(0.6, 0.2),
            Point::new(Twips::new(5), Twips::new(10)),
            Point::new(Twips::new(3), Twips::new(2)),
        ),
        (
            Matrix::scale(0.5, 0.5),
            Point::new(Twips::new(5), Twips::new(5)),
            Point::new(Twips::new(2), Twips::new(2)),
        ),
    );

    // Multiply by rotation matrices; values should be rotated around origin.
    test_multiply_twips!(
        multiply_twips_rotation,
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: Twips::ZERO,
                b: 1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            Point::new(Twips::new(10), Twips::ZERO),
            Point::new(Twips::ZERO, Twips::new(10)),
        ),
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: Twips::ZERO,
                b: 1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            PointDelta::new(Twips::new(10), Twips::ZERO),
            PointDelta::new(Twips::ZERO, Twips::new(10)),
        ),
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: Twips::ZERO,
                b: 1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            Point::new(Twips::ZERO, Twips::new(10)),
            Point::new(Twips::new(-10), Twips::ZERO),
        ),
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: Twips::ZERO,
                b: 1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            PointDelta::new(Twips::ZERO, Twips::new(10)),
            PointDelta::new(Twips::new(-10), Twips::ZERO),
        ),
        (
            Matrix {
                a: 0.0,
                c: 1.0,
                tx: Twips::ZERO,
                b: -1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            Point::new(Twips::new(10), Twips::new(10)),
            Point::new(Twips::new(10), Twips::new(-10)),
        ),
        (
            Matrix {
                a: 0.0,
                c: 1.0,
                tx: Twips::ZERO,
                b: -1.0,
                d: 0.0,
                ty: Twips::ZERO
            },
            PointDelta::new(Twips::new(10), Twips::new(10)),
            PointDelta::new(Twips::new(10), Twips::new(-10)),
        ),
        (
            Matrix::rotate(-std::f32::consts::FRAC_PI_4),
            Point::new(Twips::new(100), Twips::ZERO),
            Point::new(Twips::new(71), Twips::new(-71)),
        ),
        (
            Matrix::rotate(-std::f32::consts::FRAC_PI_4),
            PointDelta::new(Twips::new(100), Twips::ZERO),
            PointDelta::new(Twips::new(71), Twips::new(-71)),
        ),
        (
            Matrix::rotate(-std::f32::consts::FRAC_PI_4),
            Point::new(Twips::new(100), Twips::new(100)),
            Point::new(Twips::new(141), Twips::ZERO),
        ),
        (
            Matrix::rotate(-std::f32::consts::FRAC_PI_4),
            PointDelta::new(Twips::new(100), Twips::new(100)),
            PointDelta::new(Twips::new(141), Twips::ZERO),
        ),
    );

    // Testing transformation matrices that have more than 1 translation applied.
    test_multiply_twips!(
        multiply_twips_complex,
        (
            // Result of scaling by 3 * rotation by 45 degrees
            Matrix {
                a: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                c: 3.0 * std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::ZERO,
                b: 3.0 * -std::f32::consts::FRAC_PI_4.sin(),
                d: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::ZERO
            },
            Point::new(Twips::new(100), Twips::new(100)),
            Point::new(Twips::new(424), Twips::ZERO),
        ),
        (
            // Result of scaling by 3 * rotation by 45 degrees
            Matrix {
                a: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                c: 3.0 * std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::ZERO,
                b: 3.0 * -std::f32::consts::FRAC_PI_4.sin(),
                d: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::ZERO
            },
            PointDelta::new(Twips::new(100), Twips::new(100)),
            PointDelta::new(Twips::new(424), Twips::ZERO),
        ),
        (
            // Result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                c: 3.0 * std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::new(-5),
                b: 3.0 * -std::f32::consts::FRAC_PI_4.sin(),
                d: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new(5),
            },
            Point::new(Twips::new(100), Twips::new(100)),
            Point::new(Twips::new(419), Twips::new(5)),
        ),
        (
            // Result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                c: 3.0 * std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::new(-5),
                b: 3.0 * -std::f32::consts::FRAC_PI_4.sin(),
                d: 3.0 * std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new(5),
            },
            PointDelta::new(Twips::new(100), Twips::new(100)),
            PointDelta::new(Twips::new(424), Twips::ZERO),
        ),
        (
            // Result of rotation by 45 degrees * translating by (-5, 5)
            Matrix {
                a: std::f32::consts::FRAC_PI_4.cos(),
                c: std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::new(-5),
                b: -std::f32::consts::FRAC_PI_4.sin(),
                d: std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new(5),
            },
            Point::new(Twips::new(100), Twips::new(100)),
            Point::new(Twips::new(136), Twips::new(5)),
        ),
        (
            // Result of rotation by 45 degrees * translating by (-5, 5)
            Matrix {
                a: std::f32::consts::FRAC_PI_4.cos(),
                c: std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::new(-5),
                b: -std::f32::consts::FRAC_PI_4.sin(),
                d: std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new(5),
            },
            PointDelta::new(Twips::new(100), Twips::new(100)),
            PointDelta::new(Twips::new(141), Twips::ZERO),
        ),
        (
            // Result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: std::f32::consts::FRAC_PI_4.cos(),
                c: std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::ZERO,
                b: -std::f32::consts::FRAC_PI_4.sin(),
                d: std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new((10.0 * std::f32::consts::FRAC_PI_4.sin()) as i32),
            },
            Point::new(Twips::new(105), Twips::new(95)),
            Point::new(Twips::new(141), Twips::ZERO),
        ),
        (
            // Result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: std::f32::consts::FRAC_PI_4.cos(),
                c: std::f32::consts::FRAC_PI_4.sin(),
                tx: Twips::ZERO,
                b: -std::f32::consts::FRAC_PI_4.sin(),
                d: std::f32::consts::FRAC_PI_4.cos(),
                ty: Twips::new((10.0 * std::f32::consts::FRAC_PI_4.sin()) as i32),
            },
            PointDelta::new(Twips::new(105), Twips::new(95)),
            PointDelta::new(Twips::new(141), Twips::new(-7)),
        ),
    );

    #[test]
    fn test_round_to_i32() {
        assert_eq!(round_to_i32(0.0), 0);
        assert_eq!(round_to_i32(2.0), 2);
        assert_eq!(round_to_i32(2.1), 2);
        assert_eq!(round_to_i32(2.5), 2);
        assert_eq!(round_to_i32(2.9), 3);
        assert_eq!(round_to_i32(3.0), 3);
        assert_eq!(round_to_i32(3.1), 3);
        assert_eq!(round_to_i32(3.5), 4);
        assert_eq!(round_to_i32(3.9), 4);
        assert_eq!(round_to_i32(4.0), 4);
        assert_eq!(round_to_i32(-2.0), -2);
        assert_eq!(round_to_i32(-2.1), -2);
        assert_eq!(round_to_i32(-2.5), -2);
        assert_eq!(round_to_i32(-2.9), -3);
        assert_eq!(round_to_i32(-3.0), -3);
        assert_eq!(round_to_i32(-3.1), -3);
        assert_eq!(round_to_i32(-3.5), -4);
        assert_eq!(round_to_i32(-3.9), -4);
        assert_eq!(round_to_i32(-4.0), -4);
        assert_eq!(round_to_i32(f32::NAN), 0);
        assert_eq!(round_to_i32(f32::INFINITY), 0);
        assert_eq!(round_to_i32(f32::NEG_INFINITY), 0);
        assert_eq!(round_to_i32(-2147483520f32), -2147483520);
        assert_eq!(round_to_i32(-2147483648f32), i32::MIN);
        assert_eq!(round_to_i32(-2147483904f32), i32::MIN);
        assert_eq!(round_to_i32(2147483520f32), 2147483520);
        assert_eq!(round_to_i32(2147483648f32), i32::MIN);
        assert_eq!(round_to_i32(2147483904f32), i32::MIN);
    }
}
