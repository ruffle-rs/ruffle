use std::borrow::Cow;
use std::sync::Arc;

use crate::backend::{
    BitmapCacheEntry, RenderBackend, ShapeHandle, ShapeHandleImpl, ViewportDimensions,
};
use crate::bitmap::{
    Bitmap, BitmapHandle, BitmapHandleImpl, BitmapSize, BitmapSource, PixelRegion, RgbaBufRead,
    SyncHandle,
};
use crate::commands::CommandList;
use crate::error::Error;
use crate::pixel_bender::{PixelBenderShader, PixelBenderShaderArgument, PixelBenderShaderHandle};
use crate::quality::StageQuality;
use crate::shape_utils::DistilledShape;
use swf::Color;

use super::{Context3D, Context3DProfile, PixelBenderOutput, PixelBenderTarget};

pub struct NullBitmapSource;

impl BitmapSource for NullBitmapSource {
    fn bitmap_size(&self, _id: u16) -> Option<BitmapSize> {
        None
    }
    fn bitmap_handle(&self, _id: u16, _renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        None
    }
}

pub struct NullRenderer {
    dimensions: ViewportDimensions,
}

impl NullRenderer {
    pub fn new(dimensions: ViewportDimensions) -> Self {
        Self { dimensions }
    }
}

#[derive(Clone, Debug)]
struct NullBitmapHandle;
impl BitmapHandleImpl for NullBitmapHandle {}

#[derive(Clone, Debug)]
struct NullShapeHandle;
impl ShapeHandleImpl for NullShapeHandle {}

impl RenderBackend for NullRenderer {
    fn viewport_dimensions(&self) -> ViewportDimensions {
        self.dimensions
    }
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        self.dimensions = dimensions;
    }
    fn register_shape(
        &mut self,
        _shape: DistilledShape,
        _bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        ShapeHandle(Arc::new(NullShapeHandle))
    }

    fn render_offscreen(
        &mut self,
        _handle: BitmapHandle,
        _commands: CommandList,
        _quality: StageQuality,
        _bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        None
    }

    fn submit_frame(
        &mut self,
        _clear: Color,
        _commands: CommandList,
        _cache_entries: Vec<BitmapCacheEntry>,
    ) {
    }
    fn register_bitmap(&mut self, _bitmap: Bitmap) -> Result<BitmapHandle, Error> {
        Ok(BitmapHandle(Arc::new(NullBitmapHandle)))
    }

    fn update_texture(
        &mut self,
        _handle: &BitmapHandle,
        _bitmap: Bitmap,
        _region: PixelRegion,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn create_context3d(
        &mut self,
        _profile: Context3DProfile,
    ) -> Result<Box<dyn super::Context3D>, Error> {
        Err(Error::Unimplemented("createContext3D".into()))
    }

    fn context3d_present(&mut self, _context: &mut dyn Context3D) -> Result<(), Error> {
        Err(Error::Unimplemented("Context3D.present".into()))
    }

    fn debug_info(&self) -> Cow<'static, str> {
        Cow::Borrowed("Renderer: Null")
    }

    fn name(&self) -> &'static str {
        ""
    }

    fn set_quality(&mut self, _quality: StageQuality) {}

    fn run_pixelbender_shader(
        &mut self,
        _shader: PixelBenderShaderHandle,
        _arguments: &[PixelBenderShaderArgument],
        _target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, Error> {
        Err(Error::Unimplemented("Pixel bender shader".into()))
    }

    fn resolve_sync_handle(
        &mut self,
        _handle: Box<dyn SyncHandle>,
        _with_rgba: RgbaBufRead,
    ) -> Result<(), Error> {
        Err(Error::Unimplemented("Sync handle resolution".into()))
    }

    fn compile_pixelbender_shader(
        &mut self,
        _shader: PixelBenderShader,
    ) -> Result<PixelBenderShaderHandle, Error> {
        Err(Error::Unimplemented(
            "Pixel bender shader compilation".into(),
        ))
    }

    fn create_empty_texture(&mut self, _width: u32, _height: u32) -> Result<BitmapHandle, Error> {
        Ok(BitmapHandle(Arc::new(NullBitmapHandle)))
    }
}
