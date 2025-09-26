use crate::matrix::Matrix;
use crate::perspective_projection::PerspectiveProjection;
use swf::ColorTransform;

/// Represents the transform for a DisplayObject.
/// This includes both the transformation matrix and the color transform.
#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub matrix: Matrix,
    pub tz: f64,
    pub color_transform: ColorTransform,
    pub perspective_projection: Option<PerspectiveProjection>,
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
        self.0.push(Transform {
            matrix,
            color_transform,
            // TODO: Merge perspective_projections from cur_transform and transform properly
            perspective_projection: Default::default(),
            tz,
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
