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
    use approx::{assert_ulps_eq, AbsDiffEq, UlpsEq};

    macro_rules! test_invert {
        ( $test: ident, $($args: expr),* ) => {
            #[test]
            fn $test() {
                $(
                    let (mut input, output) = $args;
                    input.invert();
                    assert_ulps_eq!(input, output);
                )*
            }
        };
    }

    macro_rules! test_multiply {
        ( $test: ident, $($args: expr),* ) => {
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
        ( $test: ident, $($args: expr),* ) => {
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
        type Epsilon = <f32 as AbsDiffEq>::Epsilon;
        fn default_epsilon() -> Self::Epsilon {
            f32::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.a.abs_diff_eq(&other.a, epsilon)
                && self.b.abs_diff_eq(&other.b, epsilon)
                && self.c.abs_diff_eq(&other.c, epsilon)
                && self.d.abs_diff_eq(&other.d, epsilon)
                && self.tx.abs_diff_eq(&other.tx, epsilon)
                && self.ty.abs_diff_eq(&other.ty, epsilon)
        }
    }

    impl UlpsEq for Matrix {
        fn default_max_ulps() -> u32 {
            f32::default_max_ulps()
        }

        fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
            self.a.ulps_eq(&other.a, epsilon, max_ulps)
                && self.b.ulps_eq(&other.b, epsilon, max_ulps)
                && self.c.ulps_eq(&other.c, epsilon, max_ulps)
                && self.d.ulps_eq(&other.d, epsilon, max_ulps)
                && self.tx.ulps_eq(&other.tx, epsilon, max_ulps)
                && self.ty.ulps_eq(&other.ty, epsilon, max_ulps)
        }
    }

    // Identity matrix inverted should be unchanged
    test_invert!(
        invert_identity_matrix,
        (Matrix::default(), Matrix::default())
    );

    // Standard test cases; there's nothing special about these matrices
    test_invert!(
        invert_matrices,
        (
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: 7.0,
                b: 2.0,
                d: 5.0,
                ty: 2.0
            },
            Matrix {
                a: -1.666_666_6,
                c: 1.333_333_3,
                tx: 9.0,
                b: 0.666_666_6,
                d: -0.333_333_3,
                ty: -4.0
            }
        ),
        (
            Matrix {
                a: -1.0,
                c: -4.0,
                tx: -7.0,
                b: -2.0,
                d: -5.0,
                ty: -2.0
            },
            Matrix {
                a: 1.666_666_6,
                c: -1.333_333_3,
                tx: 9.0,
                b: -0.666_666_6,
                d: 0.333_333_3,
                ty: -4.0
            }
        ),
        (
            Matrix {
                a: 1.5,
                c: 1.2,
                tx: 1.0,
                b: -2.7,
                d: 3.4,
                ty: -2.4
            },
            Matrix {
                a: 0.407_673_9,
                c: -0.143_884_9,
                tx: -0.752_997_6,
                b: 0.323_741,
                d: 0.179_856_1,
                ty: 0.107_913_67
            }
        ),
        (
            Matrix {
                a: -2.0,
                c: 0.0,
                tx: 10.0,
                b: 0.0,
                d: -1.0,
                ty: 5.0
            },
            Matrix {
                a: -0.5,
                c: 0.0,
                tx: 5.0,
                b: 0.0,
                d: -1.0,
                ty: 5.0
            }
        )
    );

    // Anything multiplied by the identity matrix should be unchanged
    test_multiply!(
        multiply_identity_matrix,
        (Matrix::default(), Matrix::default(), Matrix::default()),
        (
            Matrix::default(),
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: 7.0,
                b: 2.0,
                d: 5.0,
                ty: 2.0
            },
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: 7.0,
                b: 2.0,
                d: 5.0,
                ty: 2.0
            }
        ),
        (
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: 7.0,
                b: 2.0,
                d: 5.0,
                ty: 2.0
            },
            Matrix::default(),
            Matrix {
                a: 1.0,
                c: 4.0,
                tx: 7.0,
                b: 2.0,
                d: 5.0,
                ty: 2.0
            }
        )
    );

    // General test cases for matrix multiplication
    test_multiply!(
        multiply_matrices,
        (
            Matrix {
                a: 6.0,
                c: 4.0,
                tx: 2.0,
                b: 5.0,
                d: 3.0,
                ty: 1.0
            },
            Matrix {
                a: 1.0,
                c: 3.0,
                tx: 5.0,
                b: 2.0,
                d: 4.0,
                ty: 6.0
            },
            Matrix {
                a: 14.0,
                c: 34.0,
                tx: 56.0,
                b: 11.0,
                d: 27.0,
                ty: 44.0
            }
        ),
        (
            Matrix {
                a: 1.0,
                c: 3.0,
                tx: 5.0,
                b: 2.0,
                d: 4.0,
                ty: 6.0
            },
            Matrix {
                a: 6.0,
                c: 4.0,
                tx: 2.0,
                b: 5.0,
                d: 3.0,
                ty: 1.0
            },
            Matrix {
                a: 21.0,
                c: 13.0,
                tx: 10.0,
                b: 32.0,
                d: 20.0,
                ty: 14.0
            }
        ),
        (
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: 3.0,
                b: 4.0,
                d: 5.0,
                ty: 6.0
            },
            Matrix {
                a: 6.0,
                c: 5.0,
                tx: 4.0,
                b: 3.0,
                d: 2.0,
                ty: 1.0
            },
            Matrix {
                a: 12.0,
                c: 9.0,
                tx: 9.0,
                b: 39.0,
                d: 30.0,
                ty: 27.0
            }
        ),
        (
            Matrix {
                a: 6.0,
                c: 5.0,
                tx: 4.0,
                b: 3.0,
                d: 2.0,
                ty: 1.0
            },
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: 3.0,
                b: 4.0,
                d: 5.0,
                ty: 6.0
            },
            Matrix {
                a: 26.0,
                c: 37.0,
                tx: 52.0,
                b: 11.0,
                d: 16.0,
                ty: 22.0
            }
        ),
        (
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: 3.0,
                b: 4.0,
                d: 5.0,
                ty: 6.0
            },
            Matrix {
                a: 1.0,
                c: 2.0,
                tx: 3.0,
                b: 4.0,
                d: 5.0,
                ty: 6.0
            },
            Matrix {
                a: 9.0,
                c: 12.0,
                tx: 18.0,
                b: 24.0,
                d: 33.0,
                ty: 48.0
            }
        )
    );

    // Twips multiplied by the identity/default matrix should be unchanged
    test_multiply_twips!(
        multiply_twips_identity_matrix,
        (
            Matrix::default(),
            (Twips::new(0), Twips::new(0)),
            (Twips::new(0), Twips::new(0))
        ),
        (
            Matrix::default(),
            (Twips::new(0), Twips::new(10)),
            (Twips::new(0), Twips::new(10))
        ),
        (
            Matrix::default(),
            (Twips::new(10), Twips::new(0)),
            (Twips::new(10), Twips::new(0))
        ),
        (
            Matrix::default(),
            (Twips::new(-251), Twips::new(152)),
            (Twips::new(-251), Twips::new(152))
        )
    );

    // multiply by translate matrices; values should be shifted
    test_multiply_twips!(
        multiply_twips_translate,
        (
            Matrix {
                a: 1.0,
                c: 0.0,
                tx: 10.0,
                b: 0.0,
                d: 1.0,
                ty: 5.0
            },
            (Twips::new(0), Twips::new(0)),
            (Twips::new(10), Twips::new(5))
        ),
        (
            Matrix {
                a: 1.0,
                c: 0.0,
                tx: -200.0,
                b: 0.0,
                d: 1.0,
                ty: 0.0
            },
            (Twips::new(50), Twips::new(20)),
            (Twips::new(-150), Twips::new(20))
        ),
        (
            Matrix {
                a: 1.0,
                c: 0.0,
                tx: 1.125,
                b: 0.0,
                d: 1.0,
                ty: 1.925
            },
            (Twips::new(0), Twips::new(0)),
            (Twips::new(1), Twips::new(1))
        )
    );

    // multiply by scalar matrices; values should be scaled up/down
    test_multiply_twips!(
        multiply_twips_scale,
        (
            Matrix {
                a: 3.0,
                c: 0.0,
                tx: 0.0,
                b: 0.0,
                d: 3.0,
                ty: 0.0
            },
            (Twips::new(0), Twips::new(0)),
            (Twips::new(0), Twips::new(0))
        ),
        (
            Matrix {
                a: 3.0,
                c: 0.0,
                tx: 0.0,
                b: 0.0,
                d: 3.0,
                ty: 0.0
            },
            (Twips::new(10), Twips::new(10)),
            (Twips::new(30), Twips::new(30))
        ),
        (
            Matrix {
                a: 0.6,
                c: 0.0,
                tx: 0.0,
                b: 0.0,
                d: 0.2,
                ty: 0.0
            },
            (Twips::new(5), Twips::new(10)),
            (Twips::new(3), Twips::new(2))
        ),
        (
            Matrix {
                a: 0.5,
                c: 0.0,
                tx: 0.0,
                b: 0.0,
                d: 0.5,
                ty: 0.0
            },
            (Twips::new(5), Twips::new(5)),
            (Twips::new(2), Twips::new(2))
        )
    );

    // multiply by rotation matrices; values should be rotated around origin
    test_multiply_twips!(
        multiply_twips_rotation,
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: 0.0,
                b: 1.0,
                d: 0.0,
                ty: 0.0
            },
            (Twips::new(10), Twips::new(0)),
            (Twips::new(0), Twips::new(10))
        ),
        (
            Matrix {
                a: 0.0,
                c: -1.0,
                tx: 0.0,
                b: 1.0,
                d: 0.0,
                ty: 0.0
            },
            (Twips::new(0), Twips::new(10)),
            (Twips::new(-10), Twips::new(0))
        ),
        (
            Matrix {
                a: 0.0,
                c: 1.0,
                tx: 0.0,
                b: -1.0,
                d: 0.0,
                ty: 0.0
            },
            (Twips::new(10), Twips::new(10)),
            (Twips::new(10), Twips::new(-10))
        ),
        (
            Matrix {
                a: f32::cos(std::f32::consts::FRAC_PI_4),
                c: f32::sin(std::f32::consts::FRAC_PI_4),
                tx: 0.0,
                b: -f32::sin(std::f32::consts::FRAC_PI_4),
                d: f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 0.0
            },
            (Twips::new(100), Twips::new(0)),
            (Twips::new(70), Twips::new(-70))
        ),
        (
            Matrix {
                a: f32::cos(std::f32::consts::FRAC_PI_4),
                c: f32::sin(std::f32::consts::FRAC_PI_4),
                tx: 0.0,
                b: -f32::sin(std::f32::consts::FRAC_PI_4),
                d: f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 0.0
            },
            (Twips::new(100), Twips::new(100)),
            (Twips::new(141), Twips::new(0))
        )
    );

    // Testing transformation matrices that have more than 1 translation applied
    test_multiply_twips!(
        multiply_twips_complex,
        (
            // result of scaling by 3 * rotation by 45 degrees
            Matrix {
                a: 3.0 * f32::cos(std::f32::consts::FRAC_PI_4),
                c: 3.0 * f32::sin(std::f32::consts::FRAC_PI_4),
                tx: 0.0,
                b: 3.0 * -f32::sin(std::f32::consts::FRAC_PI_4),
                d: 3.0 * f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 0.0
            },
            (Twips::new(100), Twips::new(100)),
            (Twips::new(424), Twips::new(0))
        ),
        (
            // result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: 3.0 * f32::cos(std::f32::consts::FRAC_PI_4),
                c: 3.0 * f32::sin(std::f32::consts::FRAC_PI_4),
                tx: -5.0,
                b: 3.0 * -f32::sin(std::f32::consts::FRAC_PI_4),
                d: 3.0 * f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 5.0
            },
            (Twips::new(100), Twips::new(100)),
            (Twips::new(419), Twips::new(5))
        ),
        (
            // result of rotation by 45 degrees * translating by (-5, 5)
            Matrix {
                a: f32::cos(std::f32::consts::FRAC_PI_4),
                c: f32::sin(std::f32::consts::FRAC_PI_4),
                tx: -5.0,
                b: -f32::sin(std::f32::consts::FRAC_PI_4),
                d: f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 5.0
            },
            (Twips::new(100), Twips::new(100)),
            (Twips::new(136), Twips::new(5))
        ),
        (
            // result of translating by (-5, 5) * rotation by 45 degrees
            Matrix {
                a: f32::cos(std::f32::consts::FRAC_PI_4),
                c: f32::sin(std::f32::consts::FRAC_PI_4),
                tx: 0.0,
                b: -f32::sin(std::f32::consts::FRAC_PI_4),
                d: f32::cos(std::f32::consts::FRAC_PI_4),
                ty: 10.0 * f32::sin(std::f32::consts::FRAC_PI_4)
            },
            (Twips::new(105), Twips::new(95)),
            (Twips::new(141), Twips::new(0))
        )
    );
}
