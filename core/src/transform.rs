use crate::prelude::*;

/// Represents the transform for a DisplayObject.
/// This includes both the transformation matrix and the color transform.
pub struct Transform {
    pub matrix: Matrix,
    pub color_transform: ColorTransform,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            matrix: Default::default(),
            color_transform: Default::default(),
        }
    }
}
