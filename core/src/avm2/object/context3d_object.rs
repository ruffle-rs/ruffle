//! Object representation for Context3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2_stub_method;
use crate::bitmap::bitmap_data::BitmapData;
use crate::context::RenderContext;
use gc_arena::{Collect, Gc, GcCell, GcWeak, Mutation};
use ruffle_render::backend::{
    BufferUsage, Context3D, Context3DBlendFactor, Context3DCommand, Context3DCompareMode,
    Context3DTextureFormat, Context3DTriangleFace, Context3DVertexBufferFormat, ProgramType,
    Texture,
};
use ruffle_render::commands::CommandHandler;
use std::cell::Cell;
use std::rc::Rc;
use swf::{Rectangle, Twips};

use super::program_3d_object::Program3DObject;
use super::texture_object::TextureObject;
use super::{ClassObject, IndexBuffer3DObject, Stage3DObject, VertexBuffer3DObject};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Context3DObject<'gc>(pub Gc<'gc, Context3DData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Context3DObjectWeak<'gc>(pub GcWeak<'gc, Context3DData<'gc>>);

impl<'gc> Context3DObject<'gc> {
    pub fn from_context(
        activation: &mut Activation<'_, 'gc>,
        context: Box<dyn Context3D>,
        stage3d: Stage3DObject<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().context3d;

        let this: Object<'gc> = Context3DObject(Gc::new(
            activation.gc(),
            Context3DData {
                base: ScriptObjectData::new(class),
                render_context: Cell::new(Some(context)),
                stage3d,
            },
        ))
        .into();

        class.call_super_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn stage3d(self) -> Stage3DObject<'gc> {
        self.0.stage3d
    }

    pub fn with_context_3d<R>(&self, f: impl FnOnce(&mut dyn Context3D) -> R) -> R {
        // Temporarily take ownership of the Context3D instance.
        let cell = &self.0.render_context;
        let mut guard = scopeguard::guard(cell.take(), |stolen| cell.set(stolen));
        f(guard
            .as_deref_mut()
            .expect("Context3D is missing or already in use"))
    }

    pub fn configure_back_buffer(
        &mut self,
        width: u32,
        height: u32,
        anti_alias: u32,
        depth_and_stencil: bool,
        wants_best_resolution: bool,
        wants_best_resolution_on_browser_zoom: bool,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::ConfigureBackBuffer {
                width,
                height,
                anti_alias,
                depth_and_stencil,
                wants_best_resolution,
                wants_best_resolution_on_browser_zoom,
            })
        });
    }

    pub fn create_index_buffer(
        &self,
        num_indices: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let index_buffer = self
            .with_context_3d(|ctx| ctx.create_index_buffer(BufferUsage::StaticDraw, num_indices));

        Ok(Value::Object(IndexBuffer3DObject::from_handle(
            activation,
            *self,
            index_buffer,
        )?))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
        class: ClassObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        check_texture_stub(activation, format);
        let texture = self.with_context_3d(|ctx| {
            ctx.create_texture(
                width,
                height,
                format,
                optimize_for_render_to_texture,
                streaming_levels,
            )
        })?;

        Ok(Value::Object(TextureObject::from_handle(
            activation, *self, texture, format, class,
        )?))
    }

    pub fn create_vertex_buffer(
        &self,
        num_vertices: u32,
        data_32_per_vertex: u8,
        usage: BufferUsage,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let handle = self.with_context_3d(|ctx| {
            ctx.create_vertex_buffer(usage, num_vertices, data_32_per_vertex)
        });
        Ok(Value::Object(VertexBuffer3DObject::from_handle(
            activation,
            *self,
            handle,
            data_32_per_vertex,
        )?))
    }

    pub fn upload_vertex_buffer_data(
        &self,
        buffer: VertexBuffer3DObject<'gc>,
        data: Vec<u8>,
        start_vertex: usize,
        data32_per_vertex: u8,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadToVertexBuffer {
                buffer: buffer.handle(),
                data,
                start_vertex,
                data32_per_vertex,
            })
        });
    }

    pub fn upload_index_buffer_data(
        &self,
        buffer: IndexBuffer3DObject<'gc>,
        data: Vec<u8>,
        start_offset: usize,
    ) {
        let mut handle = buffer.handle();
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadToIndexBuffer {
                buffer: &mut *handle,
                data,
                start_offset,
            })
        });
    }

    pub fn set_vertex_buffer_at(
        &self,
        index: u32,
        buffer: Option<(VertexBuffer3DObject<'gc>, Context3DVertexBufferFormat)>,
        buffer_offset: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetVertexBufferAt {
                index,
                buffer: buffer.map(|(b, format)| (b.handle(), format)),
                buffer_offset,
            })
        });
    }

    pub fn create_program(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Program3DObject::from_context(
            activation, *self,
        )?))
    }

    pub fn upload_shaders(
        &self,
        program: Program3DObject<'gc>,
        vertex_shader_agal: Vec<u8>,
        fragment_shader_agal: Vec<u8>,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadShaders {
                module: program.shader_module_handle(),
                vertex_shader_agal,
                fragment_shader_agal,
            })
        });
    }

    pub fn set_program(&self, program: Option<Program3DObject<'gc>>) {
        let module = program.and_then(|p| p.shader_module_handle().borrow().clone());

        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetShaders { module }));
    }

    pub fn draw_triangles(
        &self,
        index_buffer: IndexBuffer3DObject<'gc>,
        first_index: u32,
        mut num_triangles: i32,
    ) {
        if num_triangles == -1 {
            // FIXME - should we error if the number of indices isn't a multiple of 3?
            num_triangles = (index_buffer.count() / 3) as i32;
        }
        let handle = index_buffer.handle();

        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::DrawTriangles {
                index_buffer: &*handle,
                first_index: first_index as usize,
                num_triangles: num_triangles as isize,
            })
        });
    }

    pub fn set_program_constants_from_matrix(
        &self,
        program_type: ProgramType,
        first_register: u32,
        matrix_raw_data_column_major: Vec<f32>,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetProgramConstantsFromVector {
                program_type,
                first_register,
                matrix_raw_data_column_major,
            })
        });
    }

    pub fn set_culling(&self, face: Context3DTriangleFace) {
        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetCulling { face }));
    }

    pub fn set_blend_factors(
        &self,
        source_factor: Context3DBlendFactor,
        destination_factor: Context3DBlendFactor,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetBlendFactors {
                source_factor,
                destination_factor,
            })
        });
    }

    pub fn set_render_to_texture(
        &self,
        texture: Rc<dyn Texture>,
        enable_depth_and_stencil: bool,
        anti_alias: u32,
        surface_selector: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetRenderToTexture {
                texture,
                enable_depth_and_stencil,
                anti_alias,
                surface_selector,
            })
        });
    }

    pub fn set_render_to_back_buffer(&self) {
        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetRenderToBackBuffer));
    }

    pub fn present(&self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        Ok(self.with_context_3d(|ctx| activation.context.renderer.context3d_present(ctx))?)
    }

    // Renders our finalized frame to the screen, as part of the Ruffle rendering process.
    pub fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        self.with_context_3d(|context3d| {
            if context3d.should_render() {
                let handle = context3d.bitmap_handle();

                context.commands.render_stage3d(
                    handle,
                    // FIXME - apply x and y translation from Stage3D
                    context.transform_stack.transform(),
                );
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_clear(
        &self,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
        depth: f64,
        stencil: u32,
        mask: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::Clear {
                red,
                green,
                blue,
                alpha,
                depth,
                stencil,
                mask,
            })
        });
    }
    pub(crate) fn copy_bitmapdata_to_texture(
        &self,
        source: GcCell<'gc, BitmapData<'gc>>,
        dest: Rc<dyn Texture>,
        layer: u32,
    ) {
        let source = source.read();

        // Note - Flash appears to allow a source that's larger than the destination.
        // Let's leave in this assertion to see if there any real SWFS relying on this
        // behavior.
        assert!(
            source.width() <= dest.width(),
            "Source width {:?} larger than dest width {:?}",
            source.width(),
            dest.width()
        );
        assert!(
            source.height() <= dest.height(),
            "Source height {:?} larger than dest height {:?}",
            source.height(),
            dest.height()
        );

        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::CopyBitmapToTexture {
                source: source.pixels_rgba(),
                source_width: source.width(),
                source_height: source.height(),
                dest,
                layer,
            })
        });
    }

    #[cfg_attr(not(feature = "jpegxr"), allow(unused))]
    pub(crate) fn copy_pixels_to_texture(
        &self,
        source: Vec<u8>,
        dest: Rc<dyn Texture>,
        layer: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::CopyBitmapToTexture {
                source,
                source_width: dest.width(),
                source_height: dest.height(),
                dest,
                layer,
            })
        });
    }

    pub(crate) fn set_texture_at(
        &self,
        sampler: u32,
        texture: Option<Rc<dyn Texture>>,
        cube: bool,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetTextureAt {
                sampler,
                texture,
                cube,
            })
        });
    }

    pub(crate) fn set_color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetColorMask {
                red,
                green,
                blue,
                alpha,
            })
        });
    }

    pub(crate) fn set_depth_test(&self, depth_mask: bool, pass_compare_mode: Context3DCompareMode) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetDepthTest {
                depth_mask,
                pass_compare_mode,
            })
        });
    }

    pub(crate) fn create_cube_texture(
        &self,
        size: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        check_texture_stub(activation, format);
        let texture = self.with_context_3d(|ctx| {
            ctx.create_cube_texture(
                size,
                format,
                optimize_for_render_to_texture,
                streaming_levels,
            )
        })?;

        let class = activation.avm2().classes().cubetexture;

        Ok(Value::Object(TextureObject::from_handle(
            activation, *self, texture, format, class,
        )?))
    }

    pub(crate) fn set_sampler_state_at(
        &self,
        sampler: u32,
        wrap: ruffle_render::backend::Context3DWrapMode,
        filter: ruffle_render::backend::Context3DTextureFilter,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetSamplerStateAt {
                sampler,
                wrap,
                filter,
            })
        });
    }

    pub(crate) fn set_scissor_rectangle(&self, rect: Option<Rectangle<Twips>>) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetScissorRectangle { rect })
        });
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Context3DData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    #[collect(require_static)]
    render_context: Cell<Option<Box<dyn Context3D>>>,

    stage3d: Stage3DObject<'gc>,
}

const _: () = assert!(std::mem::offset_of!(Context3DData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<Context3DData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> TObject<'gc> for Context3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_context_3d(&self) -> Option<Context3DObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for Context3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context3D")
    }
}

// This would ideally be placed closer to the actual usage, but
// we don't have stub support in 'render' crates
fn check_texture_stub(activation: &mut Activation<'_, '_>, format: Context3DTextureFormat) {
    match format {
        Context3DTextureFormat::BgrPacked => {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "createTexture",
                "with BgrPacked"
            );
        }
        Context3DTextureFormat::Compressed => {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "createTexture",
                "with Compressed"
            );
        }
        _ => {}
    }
}
