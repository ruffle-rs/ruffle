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

pub struct TransformStack(Vec<Transform>);

impl TransformStack {
    pub fn new() -> Self {
        Self(vec![Transform::default()])
    }

    pub fn push(&mut self, transform: &Transform) {
        let cur_transform = self.transform();
        self.0.push(Transform {
            matrix: cur_transform.matrix * transform.matrix,
            color_transform: cur_transform.color_transform * transform.color_transform,
        });
    }

    pub fn pop(&mut self) {
        if self.0.len() <= 1 {
            panic!("Transform stack underflow");
        }
        self.0.pop();
    }

    pub fn transform(&self) -> &Transform {
        &self.0[self.0.len() - 1]
    }
}
