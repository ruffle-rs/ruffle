use super::{common::ShapeHandle, RenderBackend};
use crate::{matrix::Matrix, Color};

pub struct NullRenderer;

impl RenderBackend for NullRenderer {
    fn register_shape(&mut self, _shape: &swf::Shape) -> ShapeHandle {
        ShapeHandle(0)
    }
    fn begin_frame(&mut self) {}
    fn end_frame(&mut self) {}
    fn clear(&mut self, _color: Color) {}
    fn render_shape(&mut self, _shape: ShapeHandle, _matrix: &Matrix) {}
}
