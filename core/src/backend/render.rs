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
    fn register_bitmap_jpeg(&mut self, id: swf::CharacterId, data: &[u8], jpeg_tables: &[u8]) -> common::BitmapHandle;
    fn register_bitmap_jpeg_2(&mut self, id: swf::CharacterId, data: &[u8])
        -> common::BitmapHandle;
    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> common::BitmapHandle;

    fn begin_frame(&mut self);
    fn clear(&mut self, color: Color);
    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform);
    fn end_frame(&mut self);
}
