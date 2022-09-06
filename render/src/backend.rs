pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapInfo, BitmapSource};
use crate::error::Error;
use crate::matrix::Matrix;
use crate::shape_utils::DistilledShape;
use crate::transform::Transform;
use crate::utils;
use downcast_rs::{impl_downcast, Downcast};
use swf;

pub trait RenderBackend: Downcast {
    fn viewport_dimensions(&self) -> ViewportDimensions;
    // Do not call this method directly - use `player.set_viewport_dimensions`,
    // which will ensure that the stage is properly updated as well.
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions);
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

    /// Creates a new `RenderBackend` which renders directly
    /// to the texture specified by `BitmapHandle` with the given
    /// `width` and `height`. This backend is passed to the callback
    /// `f`, which performs the desired draw operations.
    ///
    /// After the callback `f` exectures, the texture data is copied
    /// from the GPU texture to an `RgbaImage`. There is no need to call
    /// `update_texture` with the pixels from this image, as they
    /// reflect data that is already stored on the GPU texture.
    fn render_offscreen(
        &mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        f: &mut dyn FnMut(&mut dyn RenderBackend) -> Result<(), Error>,
    ) -> Result<Bitmap, Error>;

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

    fn push_blend_mode(&mut self, blend: swf::BlendMode);
    fn pop_blend_mode(&mut self);

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap>;
    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error>;
    // Frees memory used by the bitmap. After this call, `handle` can no longer
    // be used.
    fn unregister_bitmap(&mut self, handle: BitmapHandle);
    fn update_texture(
        &mut self,
        bitmap: BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, Error>;
}
impl_downcast!(RenderBackend);

#[derive(Copy, Clone, Debug)]
pub struct ShapeHandle(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct ViewportDimensions {
    /// The dimensions of the stage's containing viewport.
    pub width: u32,
    pub height: u32,

    /// The scale factor of the containing viewport from standard-size pixels
    /// to device-scale pixels.
    pub scale_factor: f64,
}
