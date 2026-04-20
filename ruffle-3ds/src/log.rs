use ruffle_render::tessellator::ShapeTessellator;

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ruffle_render::backend::{
    BitmapCacheEntry, RenderBackend, ShapeHandle, ShapeHandleImpl, ViewportDimensions,
};
use ruffle_render::bitmap::{
    Bitmap, BitmapHandle, BitmapHandleImpl, BitmapSize, BitmapSource, PixelRegion, PixelSnapping,
    RgbaBufRead, SyncHandle,
};
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::error::Error;
use ruffle_render::pixel_bender::{
    PixelBenderShader, PixelBenderShaderArgument, PixelBenderShaderHandle,
};
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::DistilledShape;
use swf::Color;

use ruffle_render::backend::{Context3D, Context3DProfile, PixelBenderOutput, PixelBenderTarget};
use ruffle_render::matrix::Matrix;
use ruffle_render::transform::Transform;

pub struct NullBitmapSource;

impl BitmapSource for NullBitmapSource {
    fn bitmap_size(&self, _id: u16) -> Option<BitmapSize> {
        None
    }
    fn bitmap_handle(&self, _id: u16, _renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        None
    }
}

pub struct LogRenderer {
    dimensions: ViewportDimensions,
}

impl LogRenderer {
    pub fn new(dimensions: ViewportDimensions) -> Self {
        Self { dimensions }
    }
}

#[derive(Clone, Debug)]
struct NullBitmapHandle;

impl BitmapHandleImpl for NullBitmapHandle {}

#[derive(Clone, Debug)]
struct NullShapeHandle {
    id: usize,
}

static SHAPE_HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl NullShapeHandle {
    fn new() -> Self {
        Self {
            id: SHAPE_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }
}
impl ShapeHandleImpl for NullShapeHandle {}

impl RenderBackend for LogRenderer {
    fn viewport_dimensions(&self) -> ViewportDimensions {
        self.dimensions
    }
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        self.dimensions = dimensions;
    }
    fn register_shape(&mut self, shape: DistilledShape, src: &dyn BitmapSource) -> ShapeHandle {
        let string = format!("{shape:?}");
        println!("reg {}", &string[..]);

        ShapeHandle(Arc::new(NullShapeHandle::new()))
    }

    fn render_offscreen(
        &mut self,
        handle: BitmapHandle,
        commands: CommandList,
        quality: StageQuality,
        bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        println!("render offscreen");
        None
    }

    fn submit_frame(
        &mut self,
        clear: Color,
        commands: CommandList,
        cache_entries: Vec<BitmapCacheEntry>,
    ) {
        println!("clr {clear:?}");
        commands.execute(self);
    }
    fn register_bitmap(&mut self, _bitmap: Bitmap) -> Result<BitmapHandle, Error> {
        println!("register bitmap");
        Ok(BitmapHandle(Arc::new(NullBitmapHandle)))
    }

    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        region: PixelRegion,
    ) -> Result<(), Error> {
        println!("update_texture");
        Ok(())
    }

    fn create_context3d(
        &mut self,
        _profile: Context3DProfile,
    ) -> Result<Box<dyn Context3D>, Error> {
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
        println!("new empty texture");
        Ok(BitmapHandle(Arc::new(NullBitmapHandle)))
    }
}

impl CommandHandler for LogRenderer {
    fn render_bitmap(
        &mut self,
        _bitmap: BitmapHandle,
        _transform: Transform,
        _smoothing: bool,
        _pixel_snapping: PixelSnapping,
    ) {
        println!("render_bitmap called");
    }

    fn render_shape(&mut self, _shape: ShapeHandle, _transform: Transform) {
        println!("render_shape called");
    }

    fn render_stage3d(&mut self, _bitmap: BitmapHandle, _transform: Transform) {
        println!("render_stage3d called");
    }

    fn draw_rect(&mut self, _color: Color, _matrix: Matrix) {
        println!("draw_rect called");
    }

    fn draw_line(&mut self, _color: Color, _matrix: Matrix) {
        println!("draw_line called");
    }

    fn draw_line_rect(&mut self, _color: Color, _matrix: Matrix) {
        println!("draw_line_rect called");
    }

    fn push_mask(&mut self) {
        println!("push_mask called");
    }

    fn activate_mask(&mut self) {
        println!("activate_mask called");
    }

    fn deactivate_mask(&mut self) {
        println!("deactivate_mask called");
    }

    fn pop_mask(&mut self) {
        println!("pop_mask called");
    }

    fn blend(&mut self, _commands: CommandList, _blend: RenderBlendMode) {
        println!("blend called");
    }
}
