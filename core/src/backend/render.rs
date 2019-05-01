pub mod common;
#[cfg(not(target_arch = "wasm32"))]
pub mod glium;
pub mod null;
pub mod shape_utils;
#[cfg(target_arch = "wasm32")]
pub mod web_canvas;

pub use null::NullRenderer;

use self::common::ShapeHandle;
use crate::{transform::Transform, Color};

pub trait RenderBackend {
    fn set_dimensions(&mut self, width: u32, height: u32);

    fn register_shape(&mut self, shape: &swf::Shape) -> common::ShapeHandle;

    fn begin_frame(&mut self);
    fn clear(&mut self, color: Color);
    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform);
    fn end_frame(&mut self);
}
