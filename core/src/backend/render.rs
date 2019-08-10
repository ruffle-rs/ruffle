pub use crate::{transform::Transform, Color};
pub use swf;

pub trait RenderBackend {
    fn set_movie_dimensions(&mut self, width: u32, height: u32);
    fn set_viewport_dimensions(&mut self, width: u32, height: u32);
    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle;
    fn register_glyph_shape(&mut self, shape: &swf::Glyph) -> ShapeHandle;
    fn register_bitmap_jpeg(
        &mut self,
        id: swf::CharacterId,
        data: &[u8],
        jpeg_tables: &[u8],
    ) -> BitmapHandle;
    fn register_bitmap_jpeg_2(&mut self, id: swf::CharacterId, data: &[u8]) -> BitmapHandle;
    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> BitmapHandle;

    fn begin_frame(&mut self);
    fn clear(&mut self, color: Color);
    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform);
    fn end_frame(&mut self);
    fn draw_pause_overlay(&mut self);
}

#[derive(Copy, Clone, Debug)]
pub struct ShapeHandle(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct BitmapHandle(pub usize);

pub struct NullRenderer;

impl RenderBackend for NullRenderer {
    fn set_movie_dimensions(&mut self, _width: u32, _height: u32) {}
    fn set_viewport_dimensions(&mut self, _width: u32, _height: u32) {}
    fn register_shape(&mut self, _shape: &swf::Shape) -> ShapeHandle {
        ShapeHandle(0)
    }
    fn register_glyph_shape(&mut self, _shape: &swf::Glyph) -> ShapeHandle {
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
    fn draw_pause_overlay(&mut self) {}
}

pub fn glue_swf_jpeg_to_tables(jpeg_tables: &[u8], jpeg_data: &[u8]) -> Vec<u8> {
    let mut full_jpeg = Vec::with_capacity(jpeg_tables.len() + jpeg_data.len() - 4);
    full_jpeg.extend_from_slice(&jpeg_tables[..jpeg_tables.len() - 2]);
    full_jpeg.extend_from_slice(&jpeg_data[2..]);
    full_jpeg
}
