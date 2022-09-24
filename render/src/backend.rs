pub mod null;

use crate::bitmap::{Bitmap, BitmapHandle, BitmapSource, SyncHandle};
use crate::commands::CommandList;
use crate::error::Error;
use crate::filters::Filter;
use crate::quality::StageQuality;
use crate::shape_utils::DistilledShape;
use downcast_rs::{impl_downcast, Downcast};
use gc_arena::{Collect, GcCell, MutationContext};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::rc::Rc;
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
        quality: StageQuality,
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
        bitmap: &BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<(), Error>;

    fn create_context3d(&mut self) -> Result<Box<dyn Context3D>, Error>;
    fn context3d_present<'gc>(
        &mut self,
        context: &mut dyn Context3D,
        commands: Vec<Context3DCommand<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    fn debug_info(&self) -> Cow<'static, str>;

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

#[derive(Collect, Debug)]
#[collect(require_static)]
pub enum Context3DTextureFormat {
    Bgra,
    BgraPacked,
    BgrPacked,
    Compressed,
    CompressedAlpha,
    RgbaHalfFloat,
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
        vertex_size: u32,
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
    ) -> Rc<dyn Texture>;
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

    UploadToIndexBuffer {
        buffer: Rc<dyn IndexBuffer>,
        start_offset: usize,
        data: Vec<u8>,
    },

    UploadToVertexBuffer {
        buffer: Rc<dyn VertexBuffer>,
        start_vertex: usize,
        data_per_vertex: usize,
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
        source: crate::bitmap::Bitmap,
        dest: Rc<dyn Texture>,
        layer: u32,
    },
    SetTextureAt {
        sampler: u32,
        texture: Option<Rc<dyn Texture>>,
        cube: bool,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct ShapeHandle(pub usize);

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
