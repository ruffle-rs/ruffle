pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapSource};
use crate::commands::CommandList;
use crate::error::Error;
use crate::shape_utils::DistilledShape;
use downcast_rs::{impl_downcast, Downcast};
use swf;
use swf::Color;

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
        commands: CommandList,
        clear_color: Color,
    ) -> Result<Bitmap, Error>;

    fn submit_frame(&mut self, clear: swf::Color, commands: CommandList);

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
    ) -> Result<(), Error>;
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
