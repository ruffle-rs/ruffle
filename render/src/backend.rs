pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapSource, PixelRegion, RgbaBufRead, SyncHandle};
use crate::commands::CommandList;
use crate::error::Error;
use crate::filters::Filter;
use crate::pixel_bender::{PixelBenderShader, PixelBenderShaderArgument, PixelBenderShaderHandle};
use crate::quality::StageQuality;
use crate::shape_utils::DistilledShape;
use downcast_rs::{impl_downcast, Downcast};
use ruffle_wstr::WStr;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use swf::{Color, Rectangle, Twips};

pub struct BitmapCacheEntry {
    pub handle: BitmapHandle,
    pub commands: CommandList,
    pub clear: Color,
    pub filters: Vec<Filter>,
}

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

    fn is_filter_supported(&self, _filter: &Filter) -> bool {
        false
    }

    fn is_offscreen_supported(&self) -> bool {
        false
    }

    fn submit_frame(
        &mut self,
        clear: swf::Color,
        commands: CommandList,
        cache_entries: Vec<BitmapCacheEntry>,
    );

    fn create_empty_texture(&mut self, width: u32, height: u32) -> Result<BitmapHandle, Error>;

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error>;
    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        region: PixelRegion,
    ) -> Result<(), Error>;

    fn create_context3d(&mut self, profile: Context3DProfile) -> Result<Box<dyn Context3D>, Error>;
    fn context3d_present(&mut self, context: &mut dyn Context3D) -> Result<(), Error>;

    fn debug_info(&self) -> Cow<'static, str>;
    /// An internal name that is used to identify the render-backend.
    fn name(&self) -> &'static str;

    fn set_quality(&mut self, quality: StageQuality);

    fn compile_pixelbender_shader(
        &mut self,
        shader: PixelBenderShader,
    ) -> Result<PixelBenderShaderHandle, Error>;

    fn run_pixelbender_shader(
        &mut self,
        handle: PixelBenderShaderHandle,
        arguments: &[PixelBenderShaderArgument],
        target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, Error>;

    fn resolve_sync_handle(
        &mut self,
        handle: Box<dyn SyncHandle>,
        with_rgba: RgbaBufRead,
    ) -> Result<(), Error>;
}
impl_downcast!(RenderBackend);

pub enum PixelBenderTarget {
    // The shader will write to the provided bitmap texture,
    // producing a `PixelBenderOutput::Bitmap` with the corresponding
    // `SyncHandle`
    Bitmap(BitmapHandle),
    // The shader will write to a temporary texture, which will then
    // be immediately read back as bytes (in `PixelBenderOutput::Bytes`)
    Bytes { width: u32, height: u32 },
}

pub enum PixelBenderOutput {
    Bitmap(Box<dyn SyncHandle>),
    Bytes(Vec<u8>),
}

pub trait IndexBuffer: Downcast {}
impl_downcast!(IndexBuffer);
pub trait VertexBuffer: Downcast {}
impl_downcast!(VertexBuffer);

pub trait ShaderModule: Downcast {}
impl_downcast!(ShaderModule);

pub trait Texture: Downcast + Debug {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
impl_downcast!(Texture);

pub trait RawTexture: Downcast + Debug {
    fn equals(&self, other: &dyn RawTexture) -> bool;
}
impl_downcast!(RawTexture);

#[cfg(feature = "wgpu")]
impl RawTexture for wgpu::Texture {
    fn equals(&self, other: &dyn RawTexture) -> bool {
        if let Some(other_texture) = other.downcast_ref::<wgpu::Texture>() {
            std::ptr::eq(self, other_texture)
        } else {
            false
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

pub enum BufferUsage {
    DynamicDraw,
    StaticDraw,
}

pub enum ProgramType {
    Vertex,
    Fragment,
}

pub trait Context3D: Downcast {
    fn profile(&self) -> Context3DProfile;
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

    fn create_index_buffer(&mut self, usage: BufferUsage, num_indices: u32)
        -> Box<dyn IndexBuffer>;
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

    fn process_command(&mut self, command: Context3DCommand<'_>);
}
impl_downcast!(Context3D);

#[derive(Copy, Clone, Debug)]
pub enum Context3DVertexBufferFormat {
    Float1,
    Float2,
    Float3,
    Float4,
    Bytes4,
}

#[derive(Copy, Clone, Debug)]
pub enum Context3DTriangleFace {
    None,
    Back,
    Front,
    FrontAndBack,
}

#[derive(Copy, Clone, Debug)]
pub enum Context3DProfile {
    Baseline,
    BaselineConstrained,
    BaselineExtended,
    Standard,
    StandardConstrained,
    StandardExtended,
}

impl Context3DProfile {
    pub fn from_wstr(s: &WStr) -> Option<Self> {
        if s == b"baseline" {
            Some(Context3DProfile::Baseline)
        } else if s == b"baselineConstrained" {
            Some(Context3DProfile::BaselineConstrained)
        } else if s == b"baselineExtended" {
            Some(Context3DProfile::BaselineExtended)
        } else if s == b"standard" {
            Some(Context3DProfile::Standard)
        } else if s == b"standardConstrained" {
            Some(Context3DProfile::StandardConstrained)
        } else if s == b"standardExtended" {
            Some(Context3DProfile::StandardExtended)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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
pub enum Context3DCommand<'a> {
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
        buffer: &'a mut dyn IndexBuffer,
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
        index_buffer: &'a dyn IndexBuffer,
        first_index: usize,
        num_triangles: isize,
    },

    SetVertexBufferAt {
        index: u32,
        buffer: Option<(Rc<dyn VertexBuffer>, Context3DVertexBufferFormat)>,
        buffer_offset: u32,
    },

    UploadShaders {
        module: &'a RefCell<Option<Rc<dyn ShaderModule>>>,
        vertex_shader_agal: Vec<u8>,
        fragment_shader_agal: Vec<u8>,
    },

    SetShaders {
        module: Option<Rc<dyn ShaderModule>>,
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
        source: Vec<u8>,
        source_width: u32,
        source_height: u32,
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
    SetScissorRectangle {
        rect: Option<Rectangle<Twips>>,
    },
}

#[derive(Clone, Debug)]
pub struct ShapeHandle(pub Arc<dyn ShapeHandleImpl>);

pub trait ShapeHandleImpl: Downcast + Debug {}
impl_downcast!(ShapeHandleImpl);

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct ViewportDimensions {
    /// The dimensions of the stage's containing viewport.
    pub width: u32,
    pub height: u32,

    /// The scale factor of the containing viewport from standard-size pixels
    /// to device-scale pixels.
    pub scale_factor: f64,
}
