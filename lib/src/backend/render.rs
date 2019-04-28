pub mod common;
#[cfg(not(target_arch = "wasm32"))]
pub mod glium;
pub mod shape_utils;
#[cfg(target_arch = "wasm32")]
pub mod web_canvas;

use self::common::ShapeHandle;
use crate::{matrix::Matrix, Color};

pub trait RenderBackend {
    fn register_shape(&mut self, shape: &swf::Shape) -> common::ShapeHandle;

    fn begin_frame(&mut self);
    fn clear(&mut self, color: crate::Color);
    fn render_shape(&mut self, shape: ShapeHandle, matrix: &Matrix);
    fn end_frame(&mut self);
}
