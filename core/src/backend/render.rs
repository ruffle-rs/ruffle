mod null;

pub use null::{NullBitmapSource, NullRenderer};
pub use ruffle_render::utils::{determine_jpeg_tag_format, remove_invalid_jpeg_data};

use crate::shape_utils::DistilledShape;
pub use crate::transform::Transform;
use downcast_rs::Downcast;
use ruffle_render::bitmap::{Bitmap, BitmapHandle, BitmapInfo, BitmapSource};
use ruffle_render::matrix::Matrix;
use ruffle_render::utils;
pub use swf;

pub trait RenderBackend: Downcast {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32);
    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle;
    fn replace_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        handle: ShapeHandle,
    );
    fn register_glyph_shape(&mut self, shape: &swf::Glyph) -> ShapeHandle;

    fn register_bitmap_jpeg(
        &mut self,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> Result<BitmapInfo, Error> {
        let data = utils::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(&data)
    }

    fn register_bitmap_jpeg_2(&mut self, data: &[u8]) -> Result<BitmapInfo, Error> {
        let bitmap = utils::decode_define_bits_jpeg(data, None)?;
        let width = bitmap.width() as u16;
        let height = bitmap.height() as u16;
        let handle = self.register_bitmap(bitmap)?;
        Ok(BitmapInfo {
            handle,
            width,
            height,
        })
    }

    fn register_bitmap_jpeg_3_or_4(
        &mut self,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let bitmap = utils::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;
        let width = bitmap.width() as u16;
        let height = bitmap.height() as u16;
        let handle = self.register_bitmap(bitmap)?;
        Ok(BitmapInfo {
            handle,
            width,
            height,
        })
    }

    fn register_bitmap_png(
        &mut self,
        swf_tag: &swf::DefineBitsLossless,
    ) -> Result<BitmapInfo, Error> {
        let bitmap = utils::decode_define_bits_lossless(swf_tag)?;
        let width = bitmap.width() as u16;
        let height = bitmap.height() as u16;
        let handle = self.register_bitmap(bitmap)?;
        Ok(BitmapInfo {
            handle,
            width,
            height,
        })
    }

    fn begin_frame(&mut self, clear: swf::Color);
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool);
    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform);
    fn draw_rect(&mut self, color: swf::Color, matrix: &Matrix);
    fn end_frame(&mut self);
    fn push_mask(&mut self);
    fn activate_mask(&mut self);
    fn deactivate_mask(&mut self);
    fn pop_mask(&mut self);

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap>;
    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error>;
    // Frees memory used by the bitmap. After this call, `handle` can no longer
    // be used.
    fn unregister_bitmap(&mut self, handle: BitmapHandle) -> Result<(), Error>;
    fn update_texture(
        &mut self,
        bitmap: BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, Error>;
}
impl_downcast!(RenderBackend);

type Error = Box<dyn std::error::Error>;

#[derive(Copy, Clone, Debug)]
pub struct ShapeHandle(pub usize);
