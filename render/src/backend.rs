pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapSource, PixelRegion, SyncHandle};
use crate::commands::CommandList;
use crate::error::Error;
use crate::filters::Filter;
use crate::quality::StageQuality;
use crate::shape_utils::DistilledShape;
use downcast_rs::{impl_downcast, Downcast};
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_wstr::WStr;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
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

    fn render_offscreen(
        &mut self,
        handle: BitmapHandle,
        commands: CommandList,
        quality: StageQuality,
        bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>>;

    /// Applies the given filter with a `BitmapHandle` source onto a destination `BitmapHandle`.
    /// The `destination_rect` must be calculated by the caller and is assumed to be correct.
    /// Both `source_rect` and `destination_rect` must be valid (`BoundingBox::valid`).
    /// `source` may equal `destination`, in which case a temporary buffer is used internally.
    ///
    /// Returns None if the backend does not support this filter.
    fn apply_filter(
        &mut self,
        _source: BitmapHandle,
        _source_point: (u32, u32),
        _source_size: (u32, u32),
        _destination: BitmapHandle,
        _dest_point: (u32, u32),
        _filter: Filter,
    ) -> Option<Box<dyn SyncHandle>> {
        None
    }

    fn submit_frame(&mut self, clear: swf::Color, commands: CommandList);

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error>;
    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        region: PixelRegion,
    ) -> Result<(), Error>;

    fn create_context3d(&mut self) -> Result<Box<dyn Context3D>, Error>;
    fn context3d_present(&mut self, context: &mut dyn Context3D) -> Result<(), Error>;

    fn debug_info(&self) -> Cow<'static, str>;
    /// An internal name that is used to identify the render-backend.
    /// For valid values, look at:
    /// web/packages/core/src/load-options.ts:RenderBackend
    fn name(&self) -> &'static str;

    fn set_quality(&mut self, quality: StageQuality);
}
impl_downcast!(RenderBackend);

pub trait IndexBuffer: Downcast + Collect {}
impl_downcast!(IndexBuffer);
pub trait VertexBuffer: Downcast + Collect {}
impl_downcast!(VertexBuffer);

pub trait ShaderModule: Downcast + Collect {}
impl_downcast!(ShaderModule);

pub trait Texture: Downcast + Collect {}
impl_downcast!(Texture);

#[derive(Collect, Debug, Copy, Clone)]
#[collect(require_static)]
pub enum Context3DTextureFormat {
    Bgra,
    BgraPacked,
    BgrPacked,
    Compressed,
    CompressedAlpha,
    RgbaHalfFloat,
}

impl Context3DTextureFormat {
    pub fn from_wstr(wstr: &WStr) -> Option<Context3DTextureFormat> {
        if wstr == b"bgra" {
            Some(Context3DTextureFormat::Bgra)
        } else if wstr == b"bgraPacked4444" {
            Some(Context3DTextureFormat::BgraPacked)
        } else if wstr == b"bgrPacked565" {
            Some(Context3DTextureFormat::BgrPacked)
        } else if wstr == b"compressed" {
            Some(Context3DTextureFormat::Compressed)
        } else if wstr == b"compressedAlpha" {
            Some(Context3DTextureFormat::CompressedAlpha)
        } else if wstr == b"rgbaHalfFloat" {
            Some(Context3DTextureFormat::RgbaHalfFloat)
        } else {
            None
        }
    }
}

#[derive(Collect, Debug, Copy, Clone)]
#[collect(require_static)]
pub enum Context3DBlendFactor {
    DestinationAlpha,
    DestinationColor,
    One,
    OneMinusDestinationAlpha,
    OneMinusDestinationColor,
    OneMinusSourceAlpha,
    OneMinusSourceColor,
    SourceAlpha,
    SourceColor,
    Zero,
}

impl Context3DBlendFactor {
    pub fn from_wstr(wstr: &WStr) -> Option<Context3DBlendFactor> {
        if wstr == b"destinationAlpha" {
            Some(Context3DBlendFactor::DestinationAlpha)
        } else if wstr == b"destinationColor" {
            Some(Context3DBlendFactor::DestinationColor)
        } else if wstr == b"one" {
            Some(Context3DBlendFactor::One)
        } else if wstr == b"oneMinusDestinationAlpha" {
            Some(Context3DBlendFactor::OneMinusDestinationAlpha)
        } else if wstr == b"oneMinusDestinationColor" {
            Some(Context3DBlendFactor::OneMinusDestinationColor)
        } else if wstr == b"oneMinusSourceAlpha" {
            Some(Context3DBlendFactor::OneMinusSourceAlpha)
        } else if wstr == b"oneMinusSourceColor" {
            Some(Context3DBlendFactor::OneMinusSourceColor)
        } else if wstr == b"sourceAlpha" {
            Some(Context3DBlendFactor::SourceAlpha)
        } else if wstr == b"sourceColor" {
            Some(Context3DBlendFactor::SourceColor)
        } else if wstr == b"zero" {
            Some(Context3DBlendFactor::Zero)
        } else {
            None
        }
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub enum BufferUsage {
    DynamicDraw,
    StaticDraw,
}

#[derive(Collect)]
#[collect(require_static)]
pub enum ProgramType {
    Vertex,
    Fragment,
}

pub trait Context3D: Collect + Downcast {
    // The BitmapHandle for the texture we're rendering to
    fn bitmap_handle(&self) -> BitmapHandle;
    // Whether or not we should actually render the texture
    // as part of stage rendering
    fn should_render(&self) -> bool;

    // Get a 'disposed' handle - this is what we store in all IndexBuffer3D
    // objects after dispose() has been called.
    fn disposed_index_buffer_handle(&self) -> Rc<dyn IndexBuffer>;

    // Get a 'disposed' handle - this is what we store in all VertexBuffer3D
    // objects after dispose() has been called.
    fn disposed_vertex_buffer_handle(&self) -> Rc<dyn VertexBuffer>;

    fn create_index_buffer(&mut self, usage: BufferUsage, num_indices: u32) -> Rc<dyn IndexBuffer>;
    fn create_vertex_buffer(
        &mut self,
        usage: BufferUsage,
        num_vertices: u32,
        data_32_per_vertex: u8,
    ) -> Rc<dyn VertexBuffer>;

    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn Texture>, Error>;
    fn create_cube_texture(
        &mut self,
        size: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn Texture>, Error>;

    fn process_command<'gc>(
        &mut self,
        command: Context3DCommand<'gc>,
        mc: MutationContext<'gc, '_>,
    );
}
impl_downcast!(Context3D);

#[derive(Collect, Copy, Clone, Debug)]
#[collect(require_static)]
pub enum Context3DVertexBufferFormat {
    Float1,
    Float2,
    Float3,
    Float4,
    Bytes4,
}

#[derive(Collect, Copy, Clone, Debug)]
#[collect(require_static)]
pub enum Context3DTriangleFace {
    None,
    Back,
    Front,
    FrontAndBack,
}

#[derive(Collect, Copy, Clone, Debug)]
#[collect(require_static)]
pub enum Context3DCompareMode {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl Context3DCompareMode {
    pub fn from_wstr(s: &WStr) -> Option<Self> {
        if s == b"never" {
            Some(Context3DCompareMode::Never)
        } else if s == b"less" {
            Some(Context3DCompareMode::Less)
        } else if s == b"equal" {
            Some(Context3DCompareMode::Equal)
        } else if s == b"lessEqual" {
            Some(Context3DCompareMode::LessEqual)
        } else if s == b"greater" {
            Some(Context3DCompareMode::Greater)
        } else if s == b"notEqual" {
            Some(Context3DCompareMode::NotEqual)
        } else if s == b"greaterEqual" {
            Some(Context3DCompareMode::GreaterEqual)
        } else if s == b"always" {
            Some(Context3DCompareMode::Always)
        } else {
            None
        }
    }
}

#[derive(Collect, Copy, Clone, Debug)]
#[collect(require_static)]
pub enum Context3DWrapMode {
    Clamp,
    ClampURepeatV,
    Repeat,
    RepeatUClampV,
}

impl Context3DWrapMode {
    pub fn from_wstr(s: &WStr) -> Option<Self> {
        if s == b"clamp" {
            Some(Context3DWrapMode::Clamp)
        } else if s == b"clamp_u_repeat_v" {
            Some(Context3DWrapMode::ClampURepeatV)
        } else if s == b"repeat" {
            Some(Context3DWrapMode::Repeat)
        } else if s == b"repeat_u_clamp_v" {
            Some(Context3DWrapMode::RepeatUClampV)
        } else {
            None
        }
    }
}

#[derive(Collect, Copy, Clone, Debug)]
#[collect(require_static)]
pub enum Context3DTextureFilter {
    Anisotropic16X,
    Anisotropic2X,
    Anisotropic4X,
    Anisotropic8X,
    Linear,
    Nearest,
}

impl Context3DTextureFilter {
    pub fn from_wstr(s: &WStr) -> Option<Self> {
        if s == b"anisotropic16x" {
            Some(Context3DTextureFilter::Anisotropic16X)
        } else if s == b"anisotropic2x" {
            Some(Context3DTextureFilter::Anisotropic2X)
        } else if s == b"anisotropic4x" {
            Some(Context3DTextureFilter::Anisotropic4X)
        } else if s == b"anisotropic8x" {
            Some(Context3DTextureFilter::Anisotropic8X)
        } else if s == b"linear" {
            Some(Context3DTextureFilter::Linear)
        } else if s == b"nearest" {
            Some(Context3DTextureFilter::Nearest)
        } else {
            None
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub enum Context3DCommand<'gc> {
    Clear {
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
        depth: f64,
        stencil: u32,
        mask: u32,
    },
    ConfigureBackBuffer {
        width: u32,
        height: u32,
        anti_alias: u32,
        depth_and_stencil: bool,
        wants_best_resolution: bool,
        wants_best_resolution_on_browser_zoom: bool,
    },
    SetRenderToTexture {
        texture: Rc<dyn Texture>,
        enable_depth_and_stencil: bool,
        anti_alias: u32,
        surface_selector: u32,
    },
    SetRenderToBackBuffer,

    UploadToIndexBuffer {
        buffer: Rc<dyn IndexBuffer>,
        start_offset: usize,
        data: Vec<u8>,
    },

    UploadToVertexBuffer {
        buffer: Rc<dyn VertexBuffer>,
        start_vertex: usize,
        data32_per_vertex: u8,
        data: Vec<u8>,
    },

    DrawTriangles {
        index_buffer: Rc<dyn IndexBuffer>,
        first_index: usize,
        num_triangles: isize,
    },

    SetVertexBufferAt {
        index: u32,
        buffer: Option<Rc<dyn VertexBuffer>>,
        buffer_offset: u32,
        format: Context3DVertexBufferFormat,
    },

    UploadShaders {
        vertex_shader: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,
        vertex_shader_agal: Vec<u8>,
        fragment_shader: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,
        fragment_shader_agal: Vec<u8>,
    },

    SetShaders {
        vertex_shader: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,
        fragment_shader: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,
    },
    SetProgramConstantsFromVector {
        program_type: ProgramType,
        first_register: u32,
        matrix_raw_data_column_major: Vec<f32>,
    },
    SetCulling {
        face: Context3DTriangleFace,
    },
    CopyBitmapToTexture {
        source: BitmapHandle,
        dest: Rc<dyn Texture>,
        layer: u32,
    },
    SetTextureAt {
        sampler: u32,
        texture: Option<Rc<dyn Texture>>,
        cube: bool,
    },
    SetColorMask {
        red: bool,
        green: bool,
        blue: bool,
        alpha: bool,
    },
    SetDepthTest {
        depth_mask: bool,
        pass_compare_mode: Context3DCompareMode,
    },
    SetBlendFactors {
        source_factor: Context3DBlendFactor,
        destination_factor: Context3DBlendFactor,
    },
    SetSamplerStateAt {
        sampler: u32,
        wrap: Context3DWrapMode,
        filter: Context3DTextureFilter,
    },
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ShapeHandle(pub Arc<dyn ShapeHandleImpl>);

pub trait ShapeHandleImpl: Downcast + Debug {}
impl_downcast!(ShapeHandleImpl);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewportDimensions {
    /// The dimensions of the stage's containing viewport.
    pub width: u32,
    pub height: u32,

    /// The scale factor of the containing viewport from standard-size pixels
    /// to device-scale pixels.
    pub scale_factor: f64,
}
