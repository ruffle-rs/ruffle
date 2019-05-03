use super::{
    common::{BitmapHandle, ShapeHandle},
    RenderBackend,
};
use crate::{transform::Transform, Color};

pub struct NullRenderer;

impl RenderBackend for NullRenderer {
    fn set_dimensions(&mut self, _width: u32, _height: u32) {}
    fn register_shape(&mut self, _shape: &swf::Shape) -> ShapeHandle {
        ShapeHandle(0)
    }
    fn register_bitmap_jpeg(
        &mut self,
        _id: swf::CharacterId,
        _data: &[u8],
        _jpeg_tables: &[u8],
    ) -> BitmapHandle {
        BitmapHandle(0)
    }
    fn register_bitmap_jpeg_2(&mut self, _id: swf::CharacterId, _data: &[u8]) -> BitmapHandle {
        BitmapHandle(0)
    }
    fn register_bitmap_png(&mut self, _swf_tag: &swf::DefineBitsLossless) -> BitmapHandle {
        BitmapHandle(0)
    }
    fn begin_frame(&mut self) {}
    fn end_frame(&mut self) {}
    fn clear(&mut self, _color: Color) {}
    fn render_shape(&mut self, _shape: ShapeHandle, _transform: &Transform) {}
}
