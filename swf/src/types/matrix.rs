use crate::{Fixed16, Twips};
use std::ops;

/// The transformation matrix used by Flash display objects.
///
/// The matrix is a 2x3 affine transformation matrix. A point (x, y) is transformed by the matrix
/// in the following way:
/// ```text
///  [a c tx] *  [x] = [a*x + c*y + tx]
///  [b d ty]    [y]   [b*x + d*y + ty]
///  [0 0 1 ]    [1]   [1             ]
/// ```
///
/// The SWF format uses 16.16 format for `a`, `b`, `c`, `d`. Twips are used for `tx` and `ty`.
/// This means that objects in Flash can only move in units of twips, or 1/20 pixels.
///
/// [SWF19 pp.22-24](https://web.archive.org/web/20220205011833if_/https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=22)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Matrix {
    /// The matrix element at `[0, 0]`. Labeled `ScaleX` in SWF19.
    pub a: Fixed16,

    /// The matrix element at `[1, 0]`. Labeled `RotateSkew0` in SWF19.
    pub b: Fixed16,

    /// The matrix element at `[0, 1]`. Labeled `RotateSkew1` in SWF19.
    pub c: Fixed16,

    /// The matrix element at `[1, 1]`. Labeled `ScaleY` in SWF19.
    pub d: Fixed16,

    /// The X translation in twips. Labeled `TranslateX` in SWF19.
    pub tx: Twips,

    /// The Y translation in twips. Labeled `TranslateX` in SWF19.
    pub ty: Twips,
}

impl Matrix {
    /// The identity matrix.
    ///
    /// Transforming an object by this matrix has no effect.
    pub const IDENTITY: Self = Self {
        a: Fixed16::ONE,
        c: Fixed16::ZERO,
        b: Fixed16::ZERO,
        d: Fixed16::ONE,
        ty: Twips::ZERO,
        tx: Twips::ZERO,
    };

    /// Returns a scale matrix.
    #[inline]
    pub const fn scale(scale_x: Fixed16, scale_y: Fixed16) -> Self {
        Self {
            a: scale_x,
            d: scale_y,
            ..Self::IDENTITY
        }
    }

    /// Returns a rotation matrix that rotates by `angle` radians.
    #[inline]
    pub fn rotate(angle: f32) -> Self {
        Self {
            a: Fixed16::from_f32(angle.cos()),
            c: Fixed16::from_f32(-angle.sin()),
            b: Fixed16::from_f32(angle.sin()),
            d: Fixed16::from_f32(angle.cos()),
            ..Default::default()
        }
    }

    /// Returns a translation matrix.
    #[inline]
    pub const fn translate(x: Twips, y: Twips) -> Self {
        Self {
            tx: x,
            ty: y,
            ..Self::IDENTITY
        }
    }

    /// Inverts the matrix.
    ///
    /// If the matrix is not invertible, the resulting matrix will be invalid.
    #[inline]
    pub fn invert(&mut self) {
        // If we actually use this, may want to do this directly in fixed point instead of casting to f32.
        let (tx, ty) = (self.tx.get() as f32, self.ty.get() as f32);
        let a = self.a.to_f32();
        let b = self.b.to_f32();
        let c = self.c.to_f32();
        let d = self.d.to_f32();
        let det = a * d - b * c;
        let a = d / det;
        let b = b / -det;
        let c = c / -det;
        let d = a / det;
        let (out_tx, out_ty) = ((d * tx - c * ty) / -det, (b * tx - a * ty) / det);
        *self = Matrix {
            a: Fixed16::from_f32(a),
            b: Fixed16::from_f32(b),
            c: Fixed16::from_f32(c),
            d: Fixed16::from_f32(d),
            tx: Twips::new(out_tx as i32),
            ty: Twips::new(out_ty as i32),
        };
    }
}

impl ops::Mul for Matrix {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let (rhs_tx, rhs_ty) = (rhs.tx.get(), rhs.ty.get());
        let (out_tx, out_ty) = (
            self.a
                .wrapping_mul_int(rhs_tx)
                .wrapping_add(self.c.mul_int(rhs_ty))
                .wrapping_add(self.tx.get()),
            self.b
                .wrapping_mul_int(rhs_tx)
                .wrapping_add(self.d.mul_int(rhs_ty))
                .wrapping_add(self.ty.get()),
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

impl ops::Mul<(Twips, Twips)> for Matrix {
    type Output = (Twips, Twips);
    #[inline]
    fn mul(self, (x, y): (Twips, Twips)) -> (Twips, Twips) {
        let (x, y) = (x.get(), y.get());
        let out_x = (self
            .a
            .wrapping_mul_int(x)
            .wrapping_add(self.c.wrapping_mul_int(y)))
        .wrapping_add(self.tx.get());
        let out_y = (self
            .b
            .wrapping_mul_int(x)
            .wrapping_add(self.d.wrapping_mul_int(y)))
        .wrapping_add(self.ty.get());
        (Twips::new(out_x), Twips::new(out_y))
    }
}

impl Default for Matrix {
    #[inline]
    fn default() -> Matrix {
        Matrix::IDENTITY
    }
}

impl ops::MulAssign for Matrix {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
