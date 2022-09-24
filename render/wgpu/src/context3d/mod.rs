use ruffle_render::backend::{
    Context3D, Context3DCommand, Context3DTextureFormat, Context3DVertexBufferFormat, IndexBuffer,
    ProgramType, ShaderModule, VertexBuffer,
};
use ruffle_render::bitmap::{BitmapFormat, BitmapHandle};
use ruffle_render::error::Error;
use std::cell::Cell;

use wgpu::util::StagingBelt;
use wgpu::{
    BindGroup, BufferDescriptor, BufferUsages, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use wgpu::{CommandEncoder, Extent3d, RenderPass};

use crate::context3d::current_pipeline::{BoundTextureData, AGAL_FLOATS_PER_REGISTER};
use crate::descriptors::Descriptors;
use crate::Texture;
use gc_arena::{Collect, MutationContext};

use std::num::{NonZeroU32, NonZeroU64};
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

    buffer_staging_belt: StagingBelt,

    texture_view: Option<wgpu::TextureView>,

    // Note - the Context3D docs state that rendering should be double-buffered.
    // However, our Context3DCommand list already acts like a second buffer -
    // no rendering commands are actually executed until `present` is called.
    // Therefore, we only use a single texture for rendering.
    raw_texture_handle: BitmapHandle,

    current_pipeline: CurrentPipeline,
    compiled_pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<BindGroup>,

    vertex_attributes: [Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],
}

impl WgpuContext3D {
    pub fn new(descriptors: Arc<Descriptors>, raw_texture_handle: BitmapHandle) -> Self {
        // FIXME - determine the best chunk size for this
        let buffer_staging_belt = StagingBelt::new(1024);
        let current_pipeline = CurrentPipeline::new(&descriptors);

        Self {
            descriptors,
            buffer_staging_belt,
            texture_view: None,
            raw_texture_handle,
            current_pipeline,
            compiled_pipeline: None,
            bind_group: None,
            vertex_attributes: std::array::from_fn(|_| None),
        }
    }
    // Executes all of the given `commands` in response to a `Context3D.present` call.
    pub(crate) fn present<'gc>(
        &mut self,
        commands: Vec<Context3DCommand<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) {
        let mut render_command_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Context3D command encoder"),
                });
        let mut compiled_pipeline: Option<wgpu::RenderPipeline> = self.compiled_pipeline.take();
        let mut compiled_bind_group = self.bind_group.take();
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
                        tracing::warn!(
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
                        tracing::warn!(
                            "configureBackBuffer: anti_alias={anti_alias} is not yet implemented"
                        );
                    }
                    if *depth_and_stencil {
                        tracing::warn!(
                            "configureBackBuffer: depth_and_stencil is not yet implemented"
                        );
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
                                view_formats: &[format],
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::COPY_SRC
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                            });

                    finish_render_pass!(render_pass);
                    self.texture_view = Some(wgpu_texture.create_view(&Default::default()));

                    self.raw_texture_handle = BitmapHandle(Arc::new(Texture {
                        texture: Arc::new(wgpu_texture),
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        texture_offscreen: Default::default(),
                        width: *width,
                        height: *height,
                        copy_count: Cell::new(0),
                    }));
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
                        .copy_from_slice(data);
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
                        .copy_from_slice(data);
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

                    let new_pipeline = self
                        .current_pipeline
                        .rebuild_pipeline(&self.descriptors, &self.vertex_attributes);

                    if !seen_clear_command {
                        tracing::warn!("Context3D::present: drawTriangles called without first calling clear()");
                    }

                    finish_render_pass!(render_pass);

                    self.buffer_staging_belt.finish();

                    let command_buffers = [
                        // Submit the commands from the *previous* render pass first.
                        // This will be empty for the first `DrawTriangles` command in our list.
                        render_command_encoder.finish(),
                        // Then, submit all of the buffer commands we've collected.
                        buffer_command_encoder.finish(),
                    ];

                    self.descriptors.queue.submit(command_buffers);
                    self.buffer_staging_belt.recall();

                    buffer_command_encoder = self.descriptors.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: create_debug_label!("Buffer command encoder").as_deref(),
                        },
                    );

                    render_command_encoder = self.descriptors.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Context3D command encoder"),
                        },
                    );

                    // Note - we need to unconditionally re-create the render pass, since we had to submit the
                    // buffer command encoder above.

                    //if new_pipeline.is_some() || render_pass.is_none() {

                    if let Some((new_pipeline, new_bind_group)) = new_pipeline {
                        compiled_pipeline = Some(new_pipeline);
                        compiled_bind_group = Some(new_bind_group);
                    }

                    *render_pass = Some(make_render_pass(
                        self.texture_view.as_ref().unwrap(),
                        &mut render_command_encoder,
                        compiled_bind_group.as_ref().unwrap(),
                        &self.vertex_attributes,
                        // Subsequent draw calls (without an intermediate 'clear()' call)
                        // will use a clear color of None. This ensures that by itself,
                        // re-creating the render pass has no effect on the output
                        clear_color.take(),
                    ));

                    let render_pass_mut = render_pass.as_mut().unwrap();

                    render_pass_mut.set_pipeline(
                        compiled_pipeline
                            .as_ref()
                            .expect("Missing compiled pipeline"),
                    );

                    render_pass_mut
                        .set_index_buffer(index_buffer.0.slice(..), wgpu::IndexFormat::Uint16);

                    // Note - we don't submit this yet. This will be done at the end of the function (or if we hit another DrawTriangles command).
                    render_pass_mut.draw_indexed(indices, 0, 0..1);
                }

                Context3DCommand::SetVertexBufferAt {
                    index,
                    buffer,
                    buffer_offset,
                    format,
                } => {
                    let buffer = if let Some(buffer) = buffer {
                        Some(
                            buffer
                                .clone()
                                .into_any_rc()
                                .downcast::<VertexBufferWrapper>()
                                .unwrap(),
                        )
                    } else {
                        None
                    };

                    finish_render_pass!(render_pass);

                    let info = if let Some(buffer) = buffer {
                        Some(VertexAttributeInfo {
                            buffer,
                            offset_in_32bit_units: *buffer_offset as u64,
                            format: *format,
                        })
                    } else {
                        None
                    };

                    self.vertex_attributes[*index as usize] = info;
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
                        ProgramType::Vertex => &self.current_pipeline.vertex_shader_uniforms,
                        ProgramType::Fragment => &self.current_pipeline.fragment_shader_uniforms,
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
                        matrix_raw_data_column_major,
                    ));
                }
                Context3DCommand::SetCulling { face } => {
                    self.current_pipeline.set_culling(*face);
                }
                Context3DCommand::CopyBitmapToTexture {
                    source,
                    dest,
                    layer,
                } => {
                    // FIXME - handle non-BGRA8Unorm textures
                    match source.format() {
                        BitmapFormat::Rgba => {}
                        _ => unimplemented!(),
                    }

                    let bgra_data = source
                        .data()
                        .chunks_exact(4)
                        .flat_map(|chunk| {
                            // Convert from RGBA to BGRA
                            [chunk[2], chunk[1], chunk[0], chunk[3]]
                        })
                        .collect::<Vec<_>>();

                    let dest = dest.as_any().downcast_ref::<TextureWrapper>().unwrap();

                    // Note - this is very inefficient, since we allocate, and then destroy a buffer
                    // every time we copy a bitmap to a texture.
                    // Unfortunately, we cannot copy directly from the Ruffle texture, since the format
                    // is incorrect (RGBA vs BGRA).
                    // The `StagingBelt` helper doesn't support writing to textures, so we're using
                    // our own setup for now.
                    //
                    // Hopefully, writing to textures is rare enough that this isn't a big deal.

                    let texture_buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
                        label: None,
                        size: 4 * source.width() as u64 * source.height() as u64,
                        usage: BufferUsages::COPY_SRC,
                        mapped_at_creation: true,
                    });

                    let mut texture_buffer_view = texture_buffer.slice(..).get_mapped_range_mut();
                    texture_buffer_view.copy_from_slice(&bgra_data);
                    drop(texture_buffer_view);
                    texture_buffer.unmap();

                    buffer_command_encoder.copy_buffer_to_texture(
                        wgpu::ImageCopyBuffer {
                            buffer: &texture_buffer,
                            layout: wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: NonZeroU32::new(4 * source.width()),
                                rows_per_image: Some(NonZeroU32::new(source.height()).unwrap()),
                            },
                        },
                        wgpu::ImageCopyTexture {
                            texture: &dest.0,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: *layer,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: source.width(),
                            height: source.height(),
                            depth_or_array_layers: 1,
                        },
                    );
                }
                Context3DCommand::SetTextureAt {
                    sampler,
                    texture,
                    cube,
                } => {
                    finish_render_pass!(render_pass);
                    let bound_texture = if let Some(texture) = texture {
                        let texture = texture.as_any().downcast_ref::<TextureWrapper>().unwrap();

                        let mut view: wgpu::TextureViewDescriptor = Default::default();
                        if *cube {
                            view.dimension = Some(wgpu::TextureViewDimension::Cube);
                            view.array_layer_count = Some(NonZeroU32::new(6).unwrap());
                        }

                        Some(BoundTextureData {
                            view: texture.0.create_view(&view),
                            cube: *cube,
                        })
                    } else {
                        None
                    };

                    self.current_pipeline
                        .update_texture_at(*sampler as usize, bound_texture);
                }
            }
        }

        finish_render_pass!(render_pass);

        self.buffer_staging_belt.finish();

        let command_buffers = [
            // Submit the last DrawTriangles command we hit (this may be empty)
            render_command_encoder.finish(),
            // Any buffer commands were issued after the last DrawTriangles (since we
            // submit and reset the buffers after each DrawTriangles). They cannot affect
            // the current DrawTriangles, but they may update state used by future present()
            // calls.
            buffer_command_encoder.finish(),
        ];

        self.descriptors.queue.submit(command_buffers);
        self.buffer_staging_belt.recall();

        self.compiled_pipeline = compiled_pipeline;
        self.bind_group = compiled_bind_group;
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

#[derive(Collect)]
#[collect(require_static)]
pub struct TextureWrapper(wgpu::Texture);

impl IndexBuffer for IndexBufferWrapper {}
impl VertexBuffer for VertexBufferWrapper {}
impl ShaderModule for ShaderModuleAgal {}
impl ruffle_render::backend::Texture for TextureWrapper {}

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
        self.raw_texture_handle.clone()
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

    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: ruffle_render::backend::Context3DTextureFormat,
        _optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn ruffle_render::backend::Texture>, Error> {
        let format = match format {
            Context3DTextureFormat::Bgra => TextureFormat::Bgra8Unorm,
            _ => {
                return Err(Error::Unimplemented(
                    format!("Texture format {format:?}").into(),
                ))
            }
        };

        if streaming_levels != 0 {
            return Err(Error::Unimplemented(
                format!("streamingLevels={streaming_levels}").into(),
            ));
        }

        let texture = self.descriptors.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            view_formats: &[format],
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        Ok(Rc::new(TextureWrapper(texture)))
    }

    fn create_cube_texture(
        &mut self,
        size: u32,
        format: ruffle_render::backend::Context3DTextureFormat,
        _optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Rc<dyn ruffle_render::backend::Texture> {
        let format = match format {
            Context3DTextureFormat::Bgra => TextureFormat::Bgra8Unorm,
            _ => panic!("Unsupported texture format {:?}", format),
        };

        if streaming_levels != 0 {
            tracing::warn!(
                "createCubeTexture: streaming_levels={} is not yet implemented",
                streaming_levels,
            );
        }

        let texture = self.descriptors.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            view_formats: &[format],
            // Note - `optimize_for_render_to_texture` is just a hint, so
            // have to use `TextureUsages::TEXTURE_BINDING` even if the hint
            // is `false`.
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        Rc::new(TextureWrapper(texture))
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
