pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapSource, PixelRegion, RgbaBufRead, SyncHandle};
use crate::commands::CommandList;
use crate::error::Error;
use crate::filters::Filter;
use crate::pixel_bender::{PixelBenderShader, PixelBenderShaderHandle};
use crate::pixel_bender_support::PixelBenderShaderArgument;
use crate::quality::StageQuality;
use crate::shape_utils::DistilledShape;
use ruffle_wstr::{FromWStr, WStr};
use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use swf::{Color, Rectangle, Twips};

pub struct BitmapCacheEntry {
    pub handle: BitmapHandle,
    pub commands: CommandList,
    pub clear: Color,
    pub filters: Vec<Filter>,
}

pub trait RenderBackend: Any {
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
        _dest_point: (i32, i32),
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

    fn create_empty_texture(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<BitmapHandle, Error>;

    fn register_bitmap(&mut self, bitmap: Bitmap<'_>) -> Result<BitmapHandle, Error>;
    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap<'_>,
        region: PixelRegion,
    ) -> Result<(), Error>;

    fn create_context3d(&mut self, profile: Context3DProfile) -> Result<Box<dyn Context3D>, Error>;

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

pub trait IndexBuffer: Any {}
pub trait VertexBuffer: Any {}

pub trait ShaderModule: Any {}

pub trait Texture: Any + Debug {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub trait RawTexture: Any + Debug {
    fn equals(&self, other: &dyn RawTexture) -> bool;
}

#[cfg(feature = "wgpu")]
impl RawTexture for wgpu::Texture {
    fn equals(&self, other: &dyn RawTexture) -> bool {
        if let Some(other_texture) = (other as &dyn Any).downcast_ref::<wgpu::Texture>() {
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

impl FromWStr for Context3DTextureFormat {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"bgra" {
            Ok(Context3DTextureFormat::Bgra)
        } else if s == b"bgraPacked4444" {
            Ok(Context3DTextureFormat::BgraPacked)
        } else if s == b"bgrPacked565" {
            Ok(Context3DTextureFormat::BgrPacked)
        } else if s == b"compressed" {
            Ok(Context3DTextureFormat::Compressed)
        } else if s == b"compressedAlpha" {
            Ok(Context3DTextureFormat::CompressedAlpha)
        } else if s == b"rgbaHalfFloat" {
            Ok(Context3DTextureFormat::RgbaHalfFloat)
        } else {
            Err(())
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

impl FromWStr for Context3DBlendFactor {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"destinationAlpha" {
            Ok(Context3DBlendFactor::DestinationAlpha)
        } else if s == b"destinationColor" {
            Ok(Context3DBlendFactor::DestinationColor)
        } else if s == b"one" {
            Ok(Context3DBlendFactor::One)
        } else if s == b"oneMinusDestinationAlpha" {
            Ok(Context3DBlendFactor::OneMinusDestinationAlpha)
        } else if s == b"oneMinusDestinationColor" {
            Ok(Context3DBlendFactor::OneMinusDestinationColor)
        } else if s == b"oneMinusSourceAlpha" {
            Ok(Context3DBlendFactor::OneMinusSourceAlpha)
        } else if s == b"oneMinusSourceColor" {
            Ok(Context3DBlendFactor::OneMinusSourceColor)
        } else if s == b"sourceAlpha" {
            Ok(Context3DBlendFactor::SourceAlpha)
        } else if s == b"sourceColor" {
            Ok(Context3DBlendFactor::SourceColor)
        } else if s == b"zero" {
            Ok(Context3DBlendFactor::Zero)
        } else {
            Err(())
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

pub trait Context3D: Any {
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

    fn present(&mut self);
}

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

impl FromWStr for Context3DProfile {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"baseline" {
            Ok(Context3DProfile::Baseline)
        } else if s == b"baselineConstrained" {
            Ok(Context3DProfile::BaselineConstrained)
        } else if s == b"baselineExtended" {
            Ok(Context3DProfile::BaselineExtended)
        } else if s == b"standard" {
            Ok(Context3DProfile::Standard)
        } else if s == b"standardConstrained" {
            Ok(Context3DProfile::StandardConstrained)
        } else if s == b"standardExtended" {
            Ok(Context3DProfile::StandardExtended)
        } else {
            Err(())
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

impl FromWStr for Context3DCompareMode {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"never" {
            Ok(Context3DCompareMode::Never)
        } else if s == b"less" {
            Ok(Context3DCompareMode::Less)
        } else if s == b"equal" {
            Ok(Context3DCompareMode::Equal)
        } else if s == b"lessEqual" {
            Ok(Context3DCompareMode::LessEqual)
        } else if s == b"greater" {
            Ok(Context3DCompareMode::Greater)
        } else if s == b"notEqual" {
            Ok(Context3DCompareMode::NotEqual)
        } else if s == b"greaterEqual" {
            Ok(Context3DCompareMode::GreaterEqual)
        } else if s == b"always" {
            Ok(Context3DCompareMode::Always)
        } else {
            Err(())
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

impl FromWStr for Context3DWrapMode {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"clamp" {
            Ok(Context3DWrapMode::Clamp)
        } else if s == b"clamp_u_repeat_v" {
            Ok(Context3DWrapMode::ClampURepeatV)
        } else if s == b"repeat" {
            Ok(Context3DWrapMode::Repeat)
        } else if s == b"repeat_u_clamp_v" {
            Ok(Context3DWrapMode::RepeatUClampV)
        } else {
            Err(())
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

impl FromWStr for Context3DTextureFilter {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"anisotropic16x" {
            Ok(Context3DTextureFilter::Anisotropic16X)
        } else if s == b"anisotropic2x" {
            Ok(Context3DTextureFilter::Anisotropic2X)
        } else if s == b"anisotropic4x" {
            Ok(Context3DTextureFilter::Anisotropic4X)
        } else if s == b"anisotropic8x" {
            Ok(Context3DTextureFilter::Anisotropic8X)
        } else if s == b"linear" {
            Ok(Context3DTextureFilter::Linear)
        } else if s == b"nearest" {
            Ok(Context3DTextureFilter::Nearest)
        } else {
            Err(())
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
        data: &'a [u8],
    },

    UploadToVertexBuffer {
        buffer: Rc<dyn VertexBuffer>,
        start_vertex: usize,
        data32_per_vertex: u8,
        data: &'a [u8],
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
        source: &'a [u8],
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

pub trait ShapeHandleImpl: Any + Debug {}

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
