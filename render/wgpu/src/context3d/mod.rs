use ruffle_render::backend::{
    Context3D, Context3DBlendFactor, Context3DCommand, Context3DCompareMode,
    Context3DTextureFormat, Context3DVertexBufferFormat, IndexBuffer, ProgramType, Texture as _,
    VertexBuffer,
};
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::error::Error;
use std::cell::Cell;
use swf::{Rectangle, Twips};

use wgpu::util::StagingBelt;
use wgpu::{
    BindGroup, BufferDescriptor, BufferUsages, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, COPY_BUFFER_ALIGNMENT, COPY_BYTES_PER_ROW_ALIGNMENT,
};
use wgpu::{CommandEncoder, Extent3d, RenderPass};

use crate::context3d::current_pipeline::{BoundTextureData, AGAL_FLOATS_PER_REGISTER};
use crate::descriptors::Descriptors;
use crate::Texture;

use std::num::NonZeroU64;
use std::rc::Rc;
use std::sync::Arc;

mod current_pipeline;
mod shader_pair;

use current_pipeline::CurrentPipeline;

use self::shader_pair::ShaderPairAgal;

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
pub struct WgpuContext3D {
    // We only use some of the fields from `Descriptors`, but we
    // store an entire `Arc<Descriptors>` rather than wrapping the fields
    // we need in individual `Arc`s.
    descriptors: Arc<Descriptors>,

    buffer_staging_belt: StagingBelt,

    current_texture_view: Option<Rc<wgpu::TextureView>>,
    current_texture_size: Option<Extent3d>,
    current_depth_texture_view: Option<Rc<wgpu::TextureView>>,
    current_texture_resolve_view: Option<Rc<wgpu::TextureView>>,

    back_buffer_sample_count: u32,
    back_buffer_size: Option<Extent3d>,
    back_buffer_texture_view: Option<Rc<wgpu::TextureView>>,
    back_buffer_depth_texture_view: Option<Rc<wgpu::TextureView>>,
    back_buffer_resolve_texture_view: Option<Rc<wgpu::TextureView>>,

    front_buffer_texture_view: Option<Rc<wgpu::TextureView>>,
    front_buffer_depth_texture_view: Option<Rc<wgpu::TextureView>>,
    front_buffer_resolve_texture_view: Option<Rc<wgpu::TextureView>>,

    back_buffer_raw_texture_handle: BitmapHandle,
    front_buffer_raw_texture_handle: BitmapHandle,

    current_pipeline: CurrentPipeline,
    compiled_pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<BindGroup>,

    vertex_attributes: [Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],

    buffer_command_encoder: CommandEncoder,
    // We may re-create `render_pass` multiple times while processing
    // `commands`. If we do, then we only want to perform a clear if there
    // was an explicit `Clear` command. Otherwise, we want to preserve the
    // contents of the previous render pass.
    //
    // This variable gets set to `Some` when we encounter a `Clear` command,
    // and then gets set to `None` when we create a new render pass.
    clear_color: Option<ClearColor>,
    // After a call to 'present()', the Context3D API requires a call to 'clear'
    // before any new calls to 'drawTriangles'. This tracks whether we've
    // seen a `Context3DCommand::Clear` so far. Note that this is separate from
    // `clear_color`, which may be `None` even if we've seen a `Clear` command.
    seen_clear_command: bool,

    scissor_rectangle: Option<Rectangle<Twips>>,
}

impl WgpuContext3D {
    pub fn new(descriptors: Arc<Descriptors>) -> Self {
        let make_dummy_handle = || {
            let texture_label = create_debug_label!("Render target texture");
            let format = wgpu::TextureFormat::Rgba8Unorm;
            let dummy_texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
                label: texture_label.as_deref(),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                view_formats: &[format],
                usage: wgpu::TextureUsages::COPY_SRC,
            });

            BitmapHandle(Arc::new(Texture {
                bind_linear: Default::default(),
                bind_nearest: Default::default(),
                texture: Arc::new(dummy_texture),
                copy_count: Cell::new(0),
            }))
        };

        let back_buffer_raw_texture_handle = make_dummy_handle();
        let front_buffer_raw_texture_handle = make_dummy_handle();

        // FIXME - determine the best chunk size for this
        let buffer_staging_belt = StagingBelt::new(1024);
        let current_pipeline = CurrentPipeline::new(&descriptors);

        let buffer_command_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Buffer command encoder").as_deref(),
                });

        Self {
            descriptors,
            buffer_staging_belt,
            back_buffer_raw_texture_handle,
            front_buffer_raw_texture_handle,
            current_pipeline,
            compiled_pipeline: None,
            bind_group: None,
            vertex_attributes: std::array::from_fn(|_| None),

            current_texture_view: None,
            current_texture_size: None,
            current_depth_texture_view: None,
            current_texture_resolve_view: None,

            back_buffer_sample_count: 1,
            back_buffer_size: None,
            back_buffer_texture_view: None,
            back_buffer_depth_texture_view: None,
            back_buffer_resolve_texture_view: None,

            front_buffer_texture_view: None,
            front_buffer_depth_texture_view: None,
            front_buffer_resolve_texture_view: None,

            buffer_command_encoder,
            clear_color: None,
            seen_clear_command: false,
            scissor_rectangle: None,
        }
    }

    fn create_depth_texture(
        &mut self,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> Rc<TextureView> {
        Rc::new(
            self.descriptors
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Context3D depth texture"),
                    size: Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count,
                    dimension: TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                })
                .create_view(&Default::default()),
        )
    }

    // This restores rendering to our normal buffer. It can be triggered explicitly
    // from ActionScript via Context3D.setRenderToBackBuffer(), or automatically
    // when calling Context3D.present()
    fn set_render_to_back_buffer(&mut self) {
        self.current_texture_size = self.back_buffer_size;
        self.current_texture_view = self.back_buffer_texture_view.clone();
        self.current_texture_resolve_view = self.back_buffer_resolve_texture_view.clone();
        self.current_depth_texture_view = self.back_buffer_depth_texture_view.clone();
        self.current_pipeline
            .update_has_depth_texture(self.current_depth_texture_view.is_some());
        self.current_pipeline
            .update_sample_count(self.back_buffer_sample_count);
        self.current_pipeline
            .update_target_format(TextureFormat::Rgba8Unorm);
    }

    pub(crate) fn present(&mut self) {
        std::mem::swap(
            &mut self.back_buffer_raw_texture_handle,
            &mut self.front_buffer_raw_texture_handle,
        );
        std::mem::swap(
            &mut self.back_buffer_texture_view,
            &mut self.front_buffer_texture_view,
        );
        std::mem::swap(
            &mut self.back_buffer_resolve_texture_view,
            &mut self.front_buffer_resolve_texture_view,
        );
        std::mem::swap(
            &mut self.back_buffer_depth_texture_view,
            &mut self.front_buffer_depth_texture_view,
        );

        self.set_render_to_back_buffer();
        self.seen_clear_command = false;
        self.clear_color = None;
    }

    fn make_render_pass<'a>(
        &'a mut self,
        command_encoder: &'a mut CommandEncoder,
    ) -> RenderPass<'a> {
        // Subsequent draw calls (without an intermediate 'clear()' call)
        // will use a clear color of None. This ensures that by itself,
        // re-creating the render pass has no effect on the output
        let clear_color = self.clear_color.take();
        let color_load = match clear_color {
            Some(clear) if clear.mask & COLOR_MASK != 0 => wgpu::LoadOp::Clear(clear.rgb),
            _ => wgpu::LoadOp::Load,
        };

        let depth_load = match clear_color {
            Some(clear) if clear.mask & DEPTH_MASK != 0 => wgpu::LoadOp::Clear(clear.depth),
            _ => wgpu::LoadOp::Load,
        };

        let stencil_load = match clear_color {
            Some(clear) if clear.mask & STENCIL_MASK != 0 => wgpu::LoadOp::Clear(clear.stencil),
            _ => wgpu::LoadOp::Load,
        };

        let depth_stencil_attachment = if let Some(depth_view) = &self.current_depth_texture_view {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: stencil_load,
                    store: wgpu::StoreOp::Store,
                }),
            })
        } else {
            None
        };

        let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Context3D render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.current_texture_view.as_ref().unwrap(),
                resolve_target: self.current_texture_resolve_view.as_deref(),
                ops: wgpu::Operations {
                    load: color_load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            ..Default::default()
        });
        pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        pass.set_pipeline(
            self.compiled_pipeline
                .as_ref()
                .expect("Missing compiled pipeline"),
        );
        if let Some(rect) = &self.scissor_rectangle {
            let current_size = self.current_texture_size.unwrap();
            if rect.x_min.to_pixels() < 0.0
                || rect.y_min.to_pixels() < 0.0
                || rect.x_max.to_pixels() as u32 > current_size.width
                || rect.y_max.to_pixels() as u32 > current_size.height
                || rect.x_min == rect.x_max
                || rect.y_min == rect.y_max
            {
                // FIXME - throw an error when Context3D.enableErrorChecking is set
                tracing::error!(
                    "Invalid scissor rectangle {:?} for texture size {:?}",
                    rect,
                    current_size
                );
                self.scissor_rectangle = None;
            } else {
                pass.set_scissor_rect(
                    rect.x_min.to_pixels() as u32,
                    rect.y_min.to_pixels() as u32,
                    rect.width().to_pixels() as u32,
                    rect.height().to_pixels() as u32,
                );
            }
        }

        let mut seen = Vec::new();

        // Create a binding for each unique buffer that we encounter.
        // TODO - deduplicate this with the similar logic in set_pipelines
        let mut i = 0;
        for attr in self.vertex_attributes.iter().flatten() {
            if !seen.iter().any(|b| Rc::ptr_eq(b, &attr.buffer)) {
                pass.set_vertex_buffer(i as u32, attr.buffer.buffer.slice(..));
                seen.push(attr.buffer.clone());
                i += 1;
            }
        }
        pass
    }
}

pub struct IndexBufferWrapper {
    pub buffer: wgpu::Buffer,
    /// A cpu-side copy of the buffer data. This is used to allow us to
    /// perform unaligned writes to the GPU buffer, which is required by ActionScript.
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct VertexBufferWrapper {
    pub buffer: wgpu::Buffer,
    pub data_32_per_vertex: u8,
}

#[derive(Debug)]
pub struct TextureWrapper {
    texture: wgpu::Texture,
}

impl IndexBuffer for IndexBufferWrapper {}
impl VertexBuffer for VertexBufferWrapper {}
impl ruffle_render::backend::Texture for TextureWrapper {
    fn width(&self) -> u32 {
        self.texture.width()
    }
    fn height(&self) -> u32 {
        self.texture.height()
    }
}

// Context3D.setVertexBufferAt supports up to 8 vertex buffer attributes
const MAX_VERTEX_ATTRIBUTES: usize = 8;

#[derive(Clone, Debug)]
pub struct VertexAttributeInfo {
    // An offset in units of buffer entires (f32 or u8)
    offset_in_32bit_units: u64,
    format: Context3DVertexBufferFormat,
    buffer: Rc<VertexBufferWrapper>,
}

impl Context3D for WgpuContext3D {
    fn bitmap_handle(&self) -> BitmapHandle {
        self.front_buffer_raw_texture_handle.clone()
    }
    fn should_render(&self) -> bool {
        // If this is None, we haven't called configureBackBuffer yet.
        self.current_texture_view.is_some()
    }

    fn create_index_buffer(
        &mut self,
        _ruffle_usage: ruffle_render::backend::BufferUsage,
        num_indices: u32,
    ) -> Box<dyn IndexBuffer> {
        let size = align_copy_buffer_size(num_indices as usize * std::mem::size_of::<u16>()) as u32;
        let buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
            label: None,
            size: size as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Box::new(IndexBufferWrapper {
            buffer,
            data: vec![0; size as usize],
        })
    }

    fn create_vertex_buffer(
        &mut self,
        _usage: ruffle_render::backend::BufferUsage,
        num_vertices: u32,
        data_32_per_vertex: u8,
    ) -> Rc<dyn VertexBuffer> {
        let buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
            label: None,
            // Each data value is 4 bytes
            size: num_vertices as u64 * data_32_per_vertex as u64 * 4,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Rc::new(VertexBufferWrapper {
            buffer,
            data_32_per_vertex,
        })
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
        let format = convert_texture_format(format)?;

        // Wgpu doesn't support using this as a render attachment. Hopefully no swfs try
        // to use it as one.
        let render_attachment = if matches!(format, TextureFormat::Bc3RgbaUnorm) {
            TextureUsages::empty()
        } else {
            TextureUsages::RENDER_ATTACHMENT
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
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | render_attachment,
        });
        Ok(Rc::new(TextureWrapper { texture }))
    }

    fn create_cube_texture(
        &mut self,
        size: u32,
        format: ruffle_render::backend::Context3DTextureFormat,
        _optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn ruffle_render::backend::Texture>, Error> {
        let format = convert_texture_format(format)?;

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
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        });
        Ok(Rc::new(TextureWrapper { texture }))
    }

    fn process_command(&mut self, command: Context3DCommand<'_>) {
        match command {
            Context3DCommand::Clear {
                red,
                green,
                blue,
                alpha,
                depth,
                stencil,
                mask,
            } => {
                self.clear_color = Some(ClearColor {
                    rgb: wgpu::Color {
                        r: red,
                        g: green,
                        b: blue,
                        a: alpha,
                    },
                    mask,
                    depth: depth as f32,
                    stencil,
                });
                self.seen_clear_command = true;
            }
            Context3DCommand::ConfigureBackBuffer {
                width,
                height,
                anti_alias,
                depth_and_stencil,
                wants_best_resolution: _,
                wants_best_resolution_on_browser_zoom: _,
            } => {
                let mut sample_count = anti_alias;
                if sample_count == 0 {
                    sample_count = 1;
                }
                let next_pot = sample_count.next_power_of_two();
                if sample_count != next_pot {
                    // Round down to nearest power of 2
                    sample_count = next_pot / 2;
                }

                let texture_label = create_debug_label!("Render target texture");
                let format = wgpu::TextureFormat::Rgba8Unorm;

                let make_it = || {
                    // TODO - see if we can deduplicate this with the code in `CommandTarget`
                    let wgpu_texture =
                        self.descriptors
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: texture_label.as_deref(),
                                size: Extent3d {
                                    width,
                                    height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count,
                                dimension: wgpu::TextureDimension::D2,
                                format,
                                view_formats: &[format],
                                usage: if sample_count > 1 {
                                    wgpu::TextureUsages::RENDER_ATTACHMENT
                                } else {
                                    wgpu::TextureUsages::RENDER_ATTACHMENT
                                        | wgpu::TextureUsages::COPY_SRC
                                        | wgpu::TextureUsages::TEXTURE_BINDING
                                },
                            });

                    let resolve_texture = if sample_count > 1 {
                        Some(
                            self.descriptors
                                .device
                                .create_texture(&wgpu::TextureDescriptor {
                                    label: texture_label.as_deref(),
                                    size: Extent3d {
                                        width,
                                        height,
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
                                }),
                        )
                    } else {
                        None
                    };
                    (wgpu_texture, resolve_texture)
                };

                let (back_buffer_texture, back_buffer_resolve_texture) = make_it();
                let (front_buffer_texture, front_buffer_resolve_texture) = make_it();

                self.current_texture_view = Some(Rc::new(
                    back_buffer_texture.create_view(&Default::default()),
                ));

                if depth_and_stencil {
                    self.back_buffer_depth_texture_view =
                        Some(self.create_depth_texture(width, height, sample_count));
                    self.front_buffer_depth_texture_view =
                        Some(self.create_depth_texture(width, height, sample_count));
                    self.current_depth_texture_view = self.back_buffer_depth_texture_view.clone();
                } else {
                    self.back_buffer_depth_texture_view = None;
                    self.front_buffer_depth_texture_view = None;
                    self.current_depth_texture_view = None;
                }

                // Keep track of the texture/depth views, so that we can later
                // restore them in `set_render_to_back_buffer`
                self.back_buffer_sample_count = sample_count;
                self.back_buffer_size = Some(Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                });
                self.current_texture_size = self.back_buffer_size;
                self.back_buffer_texture_view = self.current_texture_view.clone();
                self.back_buffer_resolve_texture_view = back_buffer_resolve_texture
                    .as_ref()
                    .map(|t| Rc::new(t.create_view(&Default::default())));

                self.front_buffer_texture_view = Some(Rc::new(
                    front_buffer_texture.create_view(&Default::default()),
                ));
                self.front_buffer_resolve_texture_view = front_buffer_resolve_texture
                    .as_ref()
                    .map(|t| Rc::new(t.create_view(&Default::default())));

                if sample_count > 1 {
                    self.current_texture_resolve_view = Some(Rc::new(
                        back_buffer_resolve_texture
                            .as_ref()
                            .unwrap()
                            .create_view(&Default::default()),
                    ));

                    // We always use a non-multisampled texture as our raw texture handle,
                    // which is what the Stage rendering code expects. In multisample mode,
                    // this is our resolve texture.
                    self.back_buffer_raw_texture_handle = BitmapHandle(Arc::new(Texture {
                        texture: Arc::new(back_buffer_resolve_texture.unwrap()),
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        copy_count: Cell::new(0),
                    }));
                    self.front_buffer_raw_texture_handle = BitmapHandle(Arc::new(Texture {
                        texture: Arc::new(front_buffer_resolve_texture.unwrap()),
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        copy_count: Cell::new(0),
                    }));
                } else {
                    // In non-multisample mode, we don't have a separate resolve buffer,
                    // so our main texture gets used as the raw texture handle.

                    self.back_buffer_raw_texture_handle = BitmapHandle(Arc::new(Texture {
                        texture: Arc::new(back_buffer_texture),
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        copy_count: Cell::new(0),
                    }));
                    self.front_buffer_raw_texture_handle = BitmapHandle(Arc::new(Texture {
                        texture: Arc::new(front_buffer_texture),
                        bind_linear: Default::default(),
                        bind_nearest: Default::default(),
                        copy_count: Cell::new(0),
                    }));
                    self.current_texture_resolve_view = None;
                }

                self.current_pipeline
                    .update_has_depth_texture(depth_and_stencil);
                self.current_pipeline.update_sample_count(sample_count);
            }
            Context3DCommand::UploadToIndexBuffer {
                buffer,
                start_offset,
                data,
            } => {
                if data.is_empty() {
                    return;
                }
                let buffer: &mut IndexBufferWrapper = buffer
                    .as_any_mut()
                    .downcast_mut::<IndexBufferWrapper>()
                    .unwrap();

                // Unfortunately, ActionScript works with 2-byte indices, while wgpu requires
                // copy offsets and sizes to have 4-byte alignment. To support this, we need
                // to keep a copy of the data on the CPU side. We round *down* the offset to
                // the closest multiple of 4 bytes, and round *up* the length to the closest
                // multiple of 4 bytes. We then perform a copy from our CPU-side buffer, which
                // which uses the existing data (at the beiginning or end) to fill out the copy
                // to the required length and offset. Without this, we would lose data in the CPU
                // buffer whenever we performed a copy with an unalignd offset or length.
                let offset_bytes = start_offset * std::mem::size_of::<u16>();
                let rounded_down_offset =
                    offset_bytes - (offset_bytes % COPY_BUFFER_ALIGNMENT as usize);
                let rounded_up_length = align_copy_buffer_size(data.len());

                buffer.data[offset_bytes..(offset_bytes + data.len())].copy_from_slice(&data);
                self.buffer_staging_belt
                    .write_buffer(
                        &mut self.buffer_command_encoder,
                        &buffer.buffer,
                        rounded_down_offset as u64,
                        NonZeroU64::new(rounded_up_length as u64).unwrap(),
                        &self.descriptors.device,
                    )
                    .copy_from_slice(
                        &buffer.data
                            [rounded_down_offset..(rounded_down_offset + rounded_up_length)],
                    );
            }

            Context3DCommand::UploadToVertexBuffer {
                buffer,
                start_vertex,
                data32_per_vertex,
                data,
            } => {
                if data.is_empty() {
                    return;
                }

                let buffer: Rc<VertexBufferWrapper> = buffer
                    .clone()
                    .into_any_rc()
                    .downcast::<VertexBufferWrapper>()
                    .unwrap();

                // ActionScript can only work with 32-bit chunks of data, so our `write_buffer`
                // offset and size will always be a multiple of `COPY_BUFFER_ALIGNMENT` (4 bytes)
                self.buffer_staging_belt.write_buffer(
                    &mut self.buffer_command_encoder,
                    &buffer.buffer,
                    (start_vertex * (data32_per_vertex as usize) * std::mem::size_of::<f32>())
                        as u64,
                    NonZeroU64::new(data.len() as u64).unwrap(),
                    &self.descriptors.device,
                )[..data.len()]
                    .copy_from_slice(&data);
            }

            Context3DCommand::SetRenderToTexture {
                texture,
                enable_depth_and_stencil,
                anti_alias,
                surface_selector: _,
            } => {
                let mut sample_count = anti_alias;
                if sample_count == 0 {
                    sample_count = 1;
                }
                #[cfg(target_family = "wasm")]
                {
                    if sample_count > 1
                        && matches!(
                            self.descriptors.adapter.get_info().backend,
                            wgpu::Backend::Gl
                        )
                    {
                        tracing::warn!("Context.setRenderToTexture with antiAlias > 1 is not yet supported on WebGL");
                        sample_count = 1;
                    }
                }

                let texture_wrapper = texture.as_any().downcast_ref::<TextureWrapper>().unwrap();
                self.current_texture_size = Some(Extent3d {
                    width: texture_wrapper.texture.width(),
                    height: texture_wrapper.texture.height(),
                    depth_or_array_layers: 1,
                });

                if sample_count != 1 {
                    let texture_label = create_debug_label!("Render target texture MSAA");

                    let msaa_texture =
                        self.descriptors
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: texture_label.as_deref(),
                                size: Extent3d {
                                    width: texture_wrapper.texture.width(),
                                    height: texture_wrapper.texture.height(),
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count,
                                dimension: wgpu::TextureDimension::D2,
                                format: texture_wrapper.texture.format(),
                                view_formats: &[texture_wrapper.texture.format()],
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::COPY_SRC
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                            });

                    self.current_texture_resolve_view = Some(Rc::new(
                        texture_wrapper.texture.create_view(&Default::default()),
                    ));
                    self.current_texture_view =
                        Some(Rc::new(msaa_texture.create_view(&Default::default())));
                } else {
                    self.current_texture_resolve_view = None;
                    self.current_texture_view = Some(Rc::new(
                        texture_wrapper.texture.create_view(&Default::default()),
                    ));
                }

                if enable_depth_and_stencil {
                    self.current_depth_texture_view = Some(self.create_depth_texture(
                        texture_wrapper.texture.width(),
                        texture_wrapper.texture.height(),
                        sample_count,
                    ));
                } else {
                    self.current_depth_texture_view = None;
                }

                self.current_pipeline
                    .update_has_depth_texture(enable_depth_and_stencil);
                self.current_pipeline.remove_texture(&texture);
                self.current_pipeline.update_sample_count(sample_count);
                self.current_pipeline
                    .update_target_format(texture_wrapper.texture.format());
            }

            Context3DCommand::SetRenderToBackBuffer => {
                self.set_render_to_back_buffer();
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

                let indices =
                    (first_index as u32)..((first_index as u32) + (num_triangles as u32 * 3));

                let new_pipeline = self
                    .current_pipeline
                    .rebuild_pipeline(&self.descriptors, &self.vertex_attributes);

                if !self.seen_clear_command {
                    tracing::warn!(
                        "Context3D::present: drawTriangles called without first calling clear()"
                    );
                }

                self.buffer_staging_belt.finish();
                let new_encoder = self.descriptors.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: create_debug_label!("Buffer command encoder").as_deref(),
                    },
                );
                let finished_buffer_command_encoder =
                    std::mem::replace(&mut self.buffer_command_encoder, new_encoder);

                // Note - we need to unconditionally re-create the render pass, since we had to submit the
                // buffer command encoder above.

                if let Some((new_pipeline, new_bind_group)) = new_pipeline {
                    self.compiled_pipeline = Some(new_pipeline);
                    self.bind_group = Some(new_bind_group);
                }

                let mut render_command_encoder = self.descriptors.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: create_debug_label!("Render command encoder").as_deref(),
                    },
                );

                let mut render_pass = self.make_render_pass(&mut render_command_encoder);

                render_pass
                    .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(indices, 0, 0..1);

                // A `RenderPass` needs to hold references to several fields in `self`, so we can't
                // easily re-use it across multiple `DrawTriangles` calls.
                drop(render_pass);

                self.descriptors.queue.submit([
                    finished_buffer_command_encoder.finish(),
                    render_command_encoder.finish(),
                ]);
                self.buffer_staging_belt.recall();
            }

            Context3DCommand::SetVertexBufferAt {
                index,
                buffer,
                buffer_offset,
            } => {
                let info = if let Some((buffer, format)) = buffer {
                    let buffer = buffer
                        .clone()
                        .into_any_rc()
                        .downcast::<VertexBufferWrapper>()
                        .unwrap();

                    Some(VertexAttributeInfo {
                        buffer,
                        offset_in_32bit_units: buffer_offset as u64,
                        format,
                    })
                } else {
                    None
                };

                self.vertex_attributes[index as usize] = info;
                self.current_pipeline
                    .update_vertex_buffer_at(index as usize);
            }

            Context3DCommand::UploadShaders {
                module,
                vertex_shader_agal,
                fragment_shader_agal,
            } => {
                *module.borrow_mut() = Some(Rc::new(ShaderPairAgal::new(
                    vertex_shader_agal,
                    fragment_shader_agal,
                )));
            }

            Context3DCommand::SetShaders { module } => {
                let shaders =
                    module.map(|shader| shader.into_any_rc().downcast::<ShaderPairAgal>().unwrap());

                self.current_pipeline.set_shaders(shaders)
            }
            Context3DCommand::SetProgramConstantsFromVector {
                program_type,
                first_register,
                matrix_raw_data_column_major,
            } => {
                if matrix_raw_data_column_major.is_empty() {
                    return;
                }
                let buffer = match program_type {
                    ProgramType::Vertex => &self.current_pipeline.vertex_shader_uniforms,
                    ProgramType::Fragment => &self.current_pipeline.fragment_shader_uniforms,
                };

                let offset = first_register as u64
                    * AGAL_FLOATS_PER_REGISTER
                    * std::mem::size_of::<f32>() as u64;

                let mut buffer_view = self.buffer_staging_belt.write_buffer(
                    &mut self.buffer_command_encoder,
                    buffer,
                    offset,
                    NonZeroU64::new(
                        (matrix_raw_data_column_major.len() * std::mem::size_of::<f32>()) as u64,
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
                self.current_pipeline.set_culling(face);
            }
            Context3DCommand::CopyBitmapToTexture {
                mut source,
                dest,
                layer,
            } => {
                let dest = dest.as_any().downcast_ref::<TextureWrapper>().unwrap();

                // Unfortunately, we need to copy from the CPU data, rather than using the GPU texture.
                // The GPU side of a BitmapData can be updated at any time from non-Stage3D code.
                // If we were to use `self.buffer_command_encoder.copy_texture_to_texture`, the
                // BitmapData's gpu texture might be modified before we actually submit
                // `buffer_command_encoder` to the device.
                let dest_format = dest.texture.format();
                let mut bytes_per_row = dest_format.block_size(None).unwrap()
                    * (dest.width() / dest_format.block_dimensions().0);

                let rows_per_image = dest.height() / dest_format.block_dimensions().1;

                // Wgpu requires us to pad the image rows to a multiple of COPY_BYTES_PER_ROW_ALIGNMENT
                if (dest.width() * 4) % COPY_BYTES_PER_ROW_ALIGNMENT != 0
                    && matches!(dest.texture.format(), wgpu::TextureFormat::Rgba8Unorm)
                {
                    source = source
                        .chunks_exact(dest.width() as usize * 4)
                        .flat_map(|row| {
                            let padding_len = COPY_BYTES_PER_ROW_ALIGNMENT as usize
                                - (row.len() % COPY_BYTES_PER_ROW_ALIGNMENT as usize);
                            let padding = vec![0; padding_len];
                            row.iter().copied().chain(padding)
                        })
                        .collect();

                    bytes_per_row = source.len() as u32 / dest.height();
                }

                let texture_buffer = self.descriptors.device.create_buffer(&BufferDescriptor {
                    label: None,
                    size: source.len() as u64,
                    usage: BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                });

                let mut texture_buffer_view = texture_buffer.slice(..).get_mapped_range_mut();
                texture_buffer_view.copy_from_slice(&source);
                drop(texture_buffer_view);
                texture_buffer.unmap();

                self.buffer_command_encoder.copy_buffer_to_texture(
                    wgpu::ImageCopyBuffer {
                        buffer: &texture_buffer,
                        // The copy source uses the padded image data, with larger rows
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(bytes_per_row),
                            rows_per_image: Some(rows_per_image),
                        },
                    },
                    wgpu::ImageCopyTexture {
                        texture: &dest.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    // The copy size uses the orignal image, with the original row size
                    wgpu::Extent3d {
                        width: dest.width(),
                        height: dest.height(),
                        depth_or_array_layers: 1,
                    },
                );
            }
            Context3DCommand::SetTextureAt {
                sampler,
                texture,
                cube,
            } => {
                let bound_texture = if let Some(texture) = texture {
                    let texture_wrapper =
                        texture.as_any().downcast_ref::<TextureWrapper>().unwrap();

                    let mut view: wgpu::TextureViewDescriptor = Default::default();
                    if cube {
                        view.dimension = Some(wgpu::TextureViewDimension::Cube);
                        view.array_layer_count = Some(6);
                    }

                    Some(BoundTextureData {
                        id: texture.clone(),
                        view: Rc::new(texture_wrapper.texture.create_view(&view)),
                        cube,
                    })
                } else {
                    None
                };

                self.current_pipeline
                    .update_texture_at(sampler as usize, bound_texture);
            }
            Context3DCommand::SetColorMask {
                red,
                green,
                blue,
                alpha,
            } => {
                let mut color_mask = wgpu::ColorWrites::empty();
                color_mask.set(wgpu::ColorWrites::RED, red);
                color_mask.set(wgpu::ColorWrites::GREEN, green);
                color_mask.set(wgpu::ColorWrites::BLUE, blue);
                color_mask.set(wgpu::ColorWrites::ALPHA, alpha);
                self.current_pipeline.update_color_mask(color_mask);
            }
            Context3DCommand::SetDepthTest {
                depth_mask,
                pass_compare_mode,
            } => {
                let function = match pass_compare_mode {
                    Context3DCompareMode::Always => wgpu::CompareFunction::Always,
                    Context3DCompareMode::Equal => wgpu::CompareFunction::Equal,
                    Context3DCompareMode::Greater => wgpu::CompareFunction::Greater,
                    Context3DCompareMode::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
                    Context3DCompareMode::Less => wgpu::CompareFunction::Less,
                    Context3DCompareMode::LessEqual => wgpu::CompareFunction::LessEqual,
                    Context3DCompareMode::Never => wgpu::CompareFunction::Never,
                    Context3DCompareMode::NotEqual => wgpu::CompareFunction::NotEqual,
                };
                self.current_pipeline.update_depth(depth_mask, function);
            }
            Context3DCommand::SetBlendFactors {
                source_factor,
                destination_factor,
            } => {
                // This returns (color_blend_factor, alpha_blend_factor)
                let convert_blend_factor =
                    |factor: Context3DBlendFactor| -> (wgpu::BlendFactor, wgpu::BlendFactor) {
                        match factor {
                            Context3DBlendFactor::Zero => {
                                (wgpu::BlendFactor::Zero, wgpu::BlendFactor::Zero)
                            }
                            Context3DBlendFactor::One => {
                                (wgpu::BlendFactor::One, wgpu::BlendFactor::One)
                            }
                            Context3DBlendFactor::OneMinusSourceAlpha => (
                                wgpu::BlendFactor::OneMinusSrcAlpha,
                                wgpu::BlendFactor::OneMinusSrcAlpha,
                            ),
                            Context3DBlendFactor::SourceAlpha => {
                                (wgpu::BlendFactor::SrcAlpha, wgpu::BlendFactor::SrcAlpha)
                            }
                            Context3DBlendFactor::OneMinusDestinationAlpha => (
                                wgpu::BlendFactor::OneMinusDstAlpha,
                                wgpu::BlendFactor::OneMinusDstAlpha,
                            ),
                            Context3DBlendFactor::DestinationAlpha => {
                                (wgpu::BlendFactor::DstAlpha, wgpu::BlendFactor::DstAlpha)
                            }

                            Context3DBlendFactor::OneMinusSourceColor => (
                                wgpu::BlendFactor::OneMinusSrc,
                                wgpu::BlendFactor::OneMinusSrcAlpha,
                            ),
                            Context3DBlendFactor::SourceColor => {
                                (wgpu::BlendFactor::Src, wgpu::BlendFactor::SrcAlpha)
                            }
                            Context3DBlendFactor::OneMinusDestinationColor => (
                                wgpu::BlendFactor::OneMinusDst,
                                wgpu::BlendFactor::OneMinusDstAlpha,
                            ),
                            Context3DBlendFactor::DestinationColor => {
                                (wgpu::BlendFactor::Dst, wgpu::BlendFactor::DstAlpha)
                            }
                        }
                    };
                let (source_blend_factor, source_alpha_blend_factor) =
                    convert_blend_factor(source_factor);
                let (destination_blend_factor, destination_alpha_blend_factor) =
                    convert_blend_factor(destination_factor);
                // The operation is always Add for Stage3D
                self.current_pipeline.update_blend_factors(
                    wgpu::BlendComponent {
                        src_factor: source_blend_factor,
                        dst_factor: destination_blend_factor,
                        operation: wgpu::BlendOperation::Add,
                    },
                    wgpu::BlendComponent {
                        src_factor: source_alpha_blend_factor,
                        dst_factor: destination_alpha_blend_factor,
                        operation: wgpu::BlendOperation::Add,
                    },
                );
            }
            Context3DCommand::SetSamplerStateAt {
                sampler,
                wrap,
                filter,
            } => {
                self.current_pipeline
                    .update_sampler_state_at(sampler as usize, wrap, filter);
            }
            Context3DCommand::SetScissorRectangle { rect } => {
                self.scissor_rectangle = rect;
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct ClearColor {
    rgb: wgpu::Color,
    depth: f32,
    stencil: u32,
    mask: u32,
}

fn convert_texture_format(input: Context3DTextureFormat) -> Result<wgpu::TextureFormat, Error> {
    match input {
        // Some of these formats are unsupported by wgpu to various degrees:
        // * Bgra doesn't exist in webgl
        //
        // Instead, we just use Rgba8Unorm, which is the closest thing we have.
        // When we implement Texture.uploadFromByteArray, we'll need to convert
        // the user-supplied data to Rgba8Unorm.
        //
        // The Rgba8Unorm format stores more data for each channel, so this
        // will result in (hopefully minor) rendering differences.
        Context3DTextureFormat::Bgra => Ok(TextureFormat::Rgba8Unorm),
        Context3DTextureFormat::BgraPacked => Ok(TextureFormat::Rgba8Unorm),
        // Wgpu doesn't have 'Rgb8Unorm', so we use 'Rgba8Unorm' instead.
        // Applications *should* use an opaque Bitmap with this format, so the
        // alpha channel should be set to 1.0 and have no effect.
        // FIXME: Validate that this is actually the case, and throw an
        // error if we get an unexpected bitmap from ActionScript
        Context3DTextureFormat::BgrPacked => Ok(TextureFormat::Rgba8Unorm),
        // Starling claims that this is dxt5, which has an alpha channel
        Context3DTextureFormat::CompressedAlpha => Ok(TextureFormat::Bc3RgbaUnorm),
        // Starling claims that this is dxt1. It's unclear if there's supposed
        // to be an alpha channel, so we're relying on SWFS doing "the right thing"
        // as with BgrPacked
        Context3DTextureFormat::Compressed => Ok(TextureFormat::Rgba8Unorm),
        Context3DTextureFormat::RgbaHalfFloat => Ok(TextureFormat::Rgba16Float),
    }
}

// Rounds up 'len' to the nearest multiple of COPY_BUFFER_ALIGNMENT
fn align_copy_buffer_size(len: usize) -> usize {
    let align = COPY_BUFFER_ALIGNMENT as usize;
    (len + align - 1) & !(align - 1)
}
