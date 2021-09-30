use crate::html::Position;
use crate::prelude::*;
use gc_arena::Collect;

/// Represents the transform for a DisplayObject.
/// This includes both the transformation matrix and the color transform.
///
#[derive(Clone, Collect, Debug, Default)]
#[collect(require_static)]
pub struct Transform {
    pub matrix: Matrix,
    pub color_transform: ColorTransform,
}

impl From<Position<Twips>> for Transform {
    fn from(pos: Position<Twips>) -> Self {
        Self {
            matrix: Matrix {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                tx: pos.x(),
                ty: pos.y(),
            },
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
        let matrix = cur_transform.matrix * transform.matrix;
        let color_transform = cur_transform.color_transform * transform.color_transform;
        self.0.push(Transform {
            matrix,
            color_transform,
        });
    }

    pub fn pop(&mut self) {
        assert!(self.0.len() > 1, "Transform stack underflow");
        self.0.pop();
    }

    pub fn transform(&self) -> &Transform {
        &self.0[self.0.len() - 1]
    }
}

impl Default for TransformStack {
    fn default() -> Self {
        TransformStack::new()
    }
}
