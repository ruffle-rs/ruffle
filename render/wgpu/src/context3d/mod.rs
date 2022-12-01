use ruffle_render::backend::{
    Context3D, Context3DCommand, Context3DVertexBufferFormat, IndexBuffer, ProgramType,
    ShaderModule, VertexBuffer,
};
use ruffle_render::bitmap::BitmapHandle;

use wgpu::util::StagingBelt;
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindingResource, BufferDescriptor, BufferUsages,
};
use wgpu::{CommandEncoder, Extent3d, RenderPass};

use crate::descriptors::Descriptors;
use crate::Texture;
use gc_arena::{Collect, MutationContext};

use std::num::NonZeroU64;
use std::rc::Rc;
use std::sync::Arc;

mod current_pipeline;
mod render_pass_wrapper;
use render_pass_wrapper::{finish_render_pass, RenderPassWrapper};

use current_pipeline::CurrentPipeline;

const COLOR_MASK: u32 = 1 << 0;
const DEPTH_MASK: u32 = 1 << 1;
const STENCIL_MASK: u32 = 1 << 2;

/// A wgpu-based implemented of `Context3D`.
/// Many of the WGPU methods have very strict lifetime requirements
/// (e.g. taking in a reference that lives as long as the `RenderPass`).
/// As a result, most methods buffer a `Context3DCommand` without actually
/// calling any WGPU methods. The commands are then executed in `present`
///
/// The main exception to this are `create_vertex_buffer` and `create_index_buffer`.
/// These methods immediately create a `wgpu::Buffer`. This greatly simplifies
/// lifetime management - we can store an `Rc<dyn VertexBuffer>` or `Rc<dyn IndexBuffer>`
/// in the `VertexBuffer3DObject` or `IndexBuffer3DObject`. If we delayed creating them,
/// we would need to store a `GcCell<Option<Rc<dyn VertexBuffer>>>`, which prevents
/// us from obtaining a long-lived reference to the `wgpu:Bufer` (it would instead be
/// tied to the `Ref` returned by `GcCell::read`).
#[derive(Collect)]
#[collect(require_static)]
pub struct WgpuContext3D {
    // We only use some of the fields from `Descriptors`, but we
    // store an entire `Arc<Descriptors>` rather than wrapping the fields
    // we need in individual `Arc`s.
    descriptors: Arc<Descriptors>,

    // Currently, the only resources we bind are the 'program constants' uniform buffers
    // (one for the vertex shader, and one for the fragment shader).
    // These never change, so we can just create them once and reuse them.
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,

    buffer_staging_belt: StagingBelt,

    texture_view: Option<wgpu::TextureView>,

    // Note - the Context3D docs state that rendering should be double-buffered.
    // However, our Context3DCommand list already acts like a second buffer -
    // no rendering commands are actually executed until `present` is called.
    // Therefore, we only use a single texture for rendering.
    raw_texture_handle: BitmapHandle,

    vertex_shader_uniforms: wgpu::Buffer,
    fragment_shader_uniforms: wgpu::Buffer,

    current_pipeline: CurrentPipeline,
    compiled_pipeline: Option<wgpu::RenderPipeline>,

    vertex_attributes: [Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],
}

const AGAL_NUM_VERTEX_CONSTANTS: u64 = 128;
const AGAL_NUM_FRAGMENT_CONSTANTS: u64 = 28;
const AGAL_FLOATS_PER_REGISTER: u64 = 4;

const VERTEX_SHADER_UNIFORMS_BUFFER_SIZE: u64 =
    AGAL_NUM_VERTEX_CONSTANTS * AGAL_FLOATS_PER_REGISTER * std::mem::size_of::<f32>() as u64;
const FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE: u64 =
    AGAL_NUM_FRAGMENT_CONSTANTS * AGAL_FLOATS_PER_REGISTER * std::mem::size_of::<f32>() as u64;

impl WgpuContext3D {
    pub fn new(descriptors: Arc<Descriptors>, raw_texture_handle: BitmapHandle) -> Self {
        let globals_layout_label = create_debug_label!("Globals bind group layout");
        let bind_group_layout =
            descriptors
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: globals_layout_label.as_deref(),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // FIXME - determine the best chunk size for this
        let buffer_staging_belt = StagingBelt::new(1024);

        let vertex_shader_uniforms = descriptors.device.create_buffer(&BufferDescriptor {
            label: Some("Vertex shader uniforms"),
            size: VERTEX_SHADER_UNIFORMS_BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fragment_shader_uniforms = descriptors.device.create_buffer(&BufferDescriptor {
            label: Some("Fragment shader uniforms"),
            size: FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_label = create_debug_label!("Bind group");
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: bind_group_label.as_deref(),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &vertex_shader_uniforms,
                            offset: 0,
                            size: Some(
                                NonZeroU64::new(VERTEX_SHADER_UNIFORMS_BUFFER_SIZE).unwrap(),
                            ),
                        }),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &fragment_shader_uniforms,
                            offset: 0,
                            size: Some(
                                NonZeroU64::new(FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE).unwrap(),
                            ),
                        }),
                    },
                ],
            });

        Self {
            descriptors,
            bind_group_layout,
            bind_group,
            buffer_staging_belt,
            texture_view: None,
            raw_texture_handle,
            vertex_shader_uniforms,
            fragment_shader_uniforms,
            current_pipeline: CurrentPipeline::new(),
            compiled_pipeline: None,
            vertex_attributes: std::array::from_fn(|_| None),
        }
    }
    // Executes all of the given `commands` in response to a `Context3D.present` call.
    // If we needed to re-create the target Texture due to a `ConfigureBackBuffer` command,
    // then we return the new Texture.
    // If we re-used the same Texture, then we return `None`.
    pub(crate) fn present<'gc>(
        &mut self,
        commands: Vec<Context3DCommand<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Option<Texture> {
        let mut render_command_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Context3D command encoder"),
                });
        let mut compiled_pipeline: Option<wgpu::RenderPipeline> = self.compiled_pipeline.take();
        let mut render_pass = RenderPassWrapper::new(None);

        let mut buffer_command_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Buffer command encoder").as_deref(),
                });

        // We may re-create `render_pass` multiple times while processing
        // `commands`. If we do, then we only want to perform a clear if there
        // was an explicit `Clear` command. Otherwise, we want to preserve the
        // contents of the previous render pass.
        //
        // This variable gets set to `Some` when we encounter a `Clear` command,
        // and then gets set to `None` when we create a new render pass.
        let mut clear_color = None;

        // After a call to 'present()', the Context3D API requires a call to 'clear'
        // before any new calls to 'drawTriangles'. This tracks whether we've
        // seen a `Context3DCommand::Clear` so far. Note that this is separate from
        // `clear_color`, which may be `None` even if we've seen a `Clear` command.
        let mut seen_clear_command = false;

        // If we execute any `ConfigureBackBuffer` commands, this will store the newly-create
        // Texture.
        let mut recreated_texture = None;

        for command in &commands {
            match command {
                Context3DCommand::Clear {
                    red,
                    green,
                    blue,
                    alpha,
                    depth: _,
                    stencil: _,
                    mask,
                } => {
                    if *mask != COLOR_MASK | DEPTH_MASK | STENCIL_MASK {
                        log::warn!(
                            "Context3D::present: Clear command with mask {:x} not implemeneted",
                            mask
                        );
                    }

                    clear_color = Some(wgpu::Color {
                        r: *red,
                        g: *green,
                        b: *blue,
                        a: *alpha,
                    });
                    seen_clear_command = true;
                    // FIXME - clear depth and stencil buffers once we implement them

                    // Finish the current render pass - our next DrawTriangles command will create
                    // a new RenderPass using our `clear_color`.
                    finish_render_pass!(render_pass);
                }
                Context3DCommand::ConfigureBackBuffer {
                    width,
                    height,
                    anti_alias,
                    depth_and_stencil,
                    wants_best_resolution: _,
                    wants_best_resolution_on_browser_zoom: _,
                } => {
                    if *anti_alias != 1 {
                        log::warn!(
                            "configureBackBuffer: anti_alias={anti_alias} is not yet implemented"
                        );
                    }
                    if *depth_and_stencil {
                        log::warn!("configureBackBuffer: depth_and_stencil is not yet implemented");
                    }

                    let texture_label = create_debug_label!("Render target texture");
                    let format = wgpu::TextureFormat::Rgba8Unorm;

                    let wgpu_texture =
                        self.descriptors
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: texture_label.as_deref(),
                                size: Extent3d {
                                    width: *width,
                                    height: *height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::COPY_SRC
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                            });

                    finish_render_pass!(render_pass);
                    self.texture_view = Some(wgpu_texture.create_view(&Default::default()));

                    recreated_texture = Some(Texture {
                        texture: wgpu_texture,
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        texture_offscreen: None,
                        width: *width,
                        height: *height,
                    });
                }
                Context3DCommand::UploadToIndexBuffer {
                    buffer,
                    start_offset,
                    data,
                } => {
                    let buffer: &IndexBufferWrapper = buffer
                        .as_any()
                        .downcast_ref::<IndexBufferWrapper>()
                        .unwrap();

                    self.buffer_staging_belt
                        .write_buffer(
                            &mut buffer_command_encoder,
                            &buffer.0,
                            (*start_offset * std::mem::size_of::<u16>()) as u64,
                            NonZeroU64::new(data.len() as u64).unwrap(),
                            &self.descriptors.device,
                        )
                        .copy_from_slice(&data);
                }

                Context3DCommand::UploadToVertexBuffer {
                    buffer,
                    start_vertex,
                    data_per_vertex,
                    data,
                } => {
                    let buffer: Rc<VertexBufferWrapper> = buffer
                        .clone()
                        .into_any_rc()
                        .downcast::<VertexBufferWrapper>()
                        .unwrap();

                    self.buffer_staging_belt
                        .write_buffer(
                            &mut buffer_command_encoder,
                            &buffer.0,
                            (*start_vertex * *data_per_vertex * std::mem::size_of::<f32>()) as u64,
                            NonZeroU64::new(data.len() as u64).unwrap(),
                            &self.descriptors.device,
                        )
                        .copy_from_slice(&data);
                }

                Context3DCommand::DrawTriangles {
                    index_buffer,
                    first_index,
                    num_triangles,
                } => {
                    let index_buffer: &IndexBufferWrapper = index_buffer
                        .as_any()
                        .downcast_ref::<IndexBufferWrapper>()
                        .unwrap();

                    let indices = (*first_index as u32 * 3)..(*num_triangles as u32 * 3);

                    let new_pipeline = self.current_pipeline.rebuild_pipeline(
                        &self.descriptors.device,
                        &self.bind_group_layout,
                        &self.vertex_attributes,
                    );

                    if !seen_clear_command {
                        log::warn!("Context3D::present: drawTriangles called without first calling clear()");
                    }

                    if new_pipeline.is_some() || render_pass.is_none() {
                        finish_render_pass!(render_pass);

                        *render_pass = Some(make_render_pass(
                            self.texture_view.as_ref().unwrap(),
                            &mut render_command_encoder,
                            &self.bind_group,
                            &self.vertex_attributes,
                            // Subsequent draw calls (without an intermediate 'clear()' call)
                            // will use a clear color of None. This ensures that by itself,
                            // re-creating the render pass has no effect on the output
                            clear_color.take(),
                        ));

                        if let Some(new_pipeline) = new_pipeline {
                            compiled_pipeline = Some(new_pipeline);
                        }
                    }

                    let render_pass_mut = render_pass.as_mut().unwrap();

                    render_pass_mut.set_pipeline(
                        compiled_pipeline
                            .as_ref()
                            .expect("Missing compiled pipeline"),
                    );

                    render_pass_mut
                        .set_index_buffer(index_buffer.0.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass_mut.draw_indexed(indices, 0, 0..1);
                }

                Context3DCommand::SetVertexBufferAt {
                    index,
                    buffer,
                    buffer_offset,
                    format,
                } => {
                    let buffer: Rc<VertexBufferWrapper> = buffer
                        .clone()
                        .into_any_rc()
                        .downcast::<VertexBufferWrapper>()
                        .unwrap();

                    finish_render_pass!(render_pass);

                    let info = VertexAttributeInfo {
                        buffer,
                        offset_in_32bit_units: *buffer_offset as u64,
                        format: *format,
                    };
                    self.vertex_attributes[*index as usize] = Some(info);
                    self.current_pipeline
                        .update_vertex_buffer_at(*index as usize);
                }

                Context3DCommand::UploadShaders {
                    vertex_shader,
                    vertex_shader_agal,
                    fragment_shader,
                    fragment_shader_agal,
                } => {
                    *vertex_shader.write(mc) =
                        Some(Rc::new(ShaderModuleAgal(vertex_shader_agal.clone())));
                    *fragment_shader.write(mc) =
                        Some(Rc::new(ShaderModuleAgal(fragment_shader_agal.clone())));
                }

                Context3DCommand::SetShaders {
                    vertex_shader,
                    fragment_shader,
                } => {
                    let vertex_module = vertex_shader
                        .read()
                        .clone()
                        .unwrap()
                        .into_any_rc()
                        .downcast::<ShaderModuleAgal>()
                        .unwrap();
                    let fragment_module = fragment_shader
                        .read()
                        .clone()
                        .unwrap()
                        .into_any_rc()
                        .downcast::<ShaderModuleAgal>()
                        .unwrap();

                    finish_render_pass!(render_pass);

                    self.current_pipeline
                        .set_vertex_shader(vertex_module.clone());
                    self.current_pipeline
                        .set_fragment_shader(fragment_module.clone());
                }
                Context3DCommand::SetProgramConstantsFromVector {
                    program_type,
                    first_register,
                    matrix_raw_data_column_major,
                } => {
                    let buffer = match program_type {
                        ProgramType::Vertex => &self.vertex_shader_uniforms,
                        ProgramType::Fragment => &self.fragment_shader_uniforms,
                    };

                    let offset = *first_register as u64
                        * AGAL_FLOATS_PER_REGISTER
                        * std::mem::size_of::<f32>() as u64;

                    let mut buffer_view = self.buffer_staging_belt.write_buffer(
                        &mut buffer_command_encoder,
                        buffer,
                        offset,
                        NonZeroU64::new(
                            (matrix_raw_data_column_major.len() * std::mem::size_of::<f32>())
                                as u64,
                        )
                        .unwrap(),
                        &self.descriptors.device,
                    );
                    // Despite what the docs claim, we copy in *column* major order, rather than *row* major order.
                    // See this code in OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/display3D/Context3D.hx#L1532-L1550
                    // When the 'transposedMatrix' flag is false, it copies data *directly* from matrix.rawData,
                    // which is stored in column-major order
                    buffer_view.copy_from_slice(bytemuck::cast_slice::<f32, u8>(
                        &matrix_raw_data_column_major,
                    ));
                }
                Context3DCommand::SetCulling { face } => {
                    self.current_pipeline.set_culling(*face);
                }
            }
        }

        finish_render_pass!(render_pass);

        self.buffer_staging_belt.finish();

        let command_buffers = vec![
            buffer_command_encoder.finish(),
            render_command_encoder.finish(),
        ];

        self.descriptors.queue.submit(command_buffers);
        self.buffer_staging_belt.recall();

        self.compiled_pipeline = compiled_pipeline;

        recreated_texture
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub struct IndexBufferWrapper(wgpu::Buffer);

#[derive(Collect)]
#[collect(require_static)]
pub struct VertexBufferWrapper(wgpu::Buffer);

#[derive(Collect)]
#[collect(require_static)]
pub struct ShaderModuleAgal(Vec<u8>);

impl IndexBuffer for IndexBufferWrapper {}
impl VertexBuffer for VertexBufferWrapper {}
impl ShaderModule for ShaderModuleAgal {}

// Context3D.setVertexBufferAt supports up to 8 vertex buffer attributes
const MAX_VERTEX_ATTRIBUTES: usize = 8;

#[derive(Clone)]
pub struct VertexAttributeInfo {
    // An offset in units of buffer entires (f32 or u8)
    offset_in_32bit_units: u64,
    format: Context3DVertexBufferFormat,
    buffer: Rc<VertexBufferWrapper>,
}

impl Context3D for WgpuContext3D {
    fn bitmap_handle(&self) -> BitmapHandle {
        self.raw_texture_handle
    }
    fn should_render(&self) -> bool {
        // If this is None, we haven't called configureBackBuffer yet.
        self.texture_view.is_some()
    }

    fn create_index_buffer(
        &mut self,
        _ruffle_usage: ruffle_render::backend::BufferUsage,
        num_indices: u32,
    ) -> Rc<dyn IndexBuffer> {
        let buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
            label: None,
            size: num_indices as u64 * 2,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Rc::new(IndexBufferWrapper(buffer))
    }

    fn create_vertex_buffer(
        &mut self,
        _usage: ruffle_render::backend::BufferUsage,
        num_vertices: u32,
        data_per_vertex: u32,
    ) -> Rc<dyn VertexBuffer> {
        let buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
            label: None,
            // Each data value is 4 bytes
            size: num_vertices as u64 * data_per_vertex as u64 * 4,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Rc::new(VertexBufferWrapper(buffer))
    }

    fn disposed_index_buffer_handle(&self) -> Rc<dyn IndexBuffer> {
        todo!()
    }

    fn disposed_vertex_buffer_handle(&self) -> Rc<dyn VertexBuffer> {
        todo!()
    }
}

// This cannot be a method on `self`, because we need to only borrow certain fields
// with the long lifetime 'a
fn make_render_pass<'a>(
    texture_view: &'a wgpu::TextureView,
    command_encoder: &'a mut CommandEncoder,
    bind_group: &'a BindGroup,
    vertex_attributes: &'a [Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],
    clear_color: Option<wgpu::Color>,
) -> RenderPass<'a> {
    let load = match clear_color {
        Some(_) => wgpu::LoadOp::Clear(clear_color.unwrap()),
        None => wgpu::LoadOp::Load,
    };

    let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Context3D render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            ops: wgpu::Operations { load, store: true },
        })],
        depth_stencil_attachment: None,
    });
    pass.set_bind_group(0, bind_group, &[]);
    for (i, attr) in vertex_attributes.iter().enumerate() {
        if let Some(attr) = attr {
            pass.set_vertex_buffer(i as u32, attr.buffer.0.slice(..));
        }
    }
    pass
}
