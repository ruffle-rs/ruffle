use crate::descriptors::DescriptorsTargetData;
use crate::target::RenderTargetFrame;
use crate::target::TextureTarget;
use crate::{
    create_buffer_with_data, create_quad_buffers, format_list, get_backend_names, target,
    BufferDimensions, ColorAdjustments, Descriptors, Draw, DrawType, Error, Frame, Globals,
    GradientStorage, GradientUniforms, MaskState, Mesh, RegistryData, RenderTarget,
    SwapChainTarget, Texture, TextureOffscreen, TextureTransforms, Transforms, UniformBuffer,
    Vertex,
};
use fnv::FnvHashMap;
use ruffle_render::backend::{RenderBackend, ShapeHandle, ViewportDimensions};
use ruffle_render::bitmap::{Bitmap, BitmapHandle, BitmapSource};
use ruffle_render::commands::{CommandHandler, CommandList};
use ruffle_render::error::Error as BitmapError;
use ruffle_render::shape_utils::DistilledShape;
use ruffle_render::tessellator::{DrawType as TessDrawType, ShapeTessellator};
use ruffle_render::transform::Transform;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use swf::{BlendMode, Color};

const DEFAULT_SAMPLE_COUNT: u32 = 4;

pub struct WgpuRenderBackend<T: RenderTarget> {
    descriptors: Arc<Descriptors>,
    globals: Globals,
    uniform_buffers: UniformBuffer<Transforms>,
    target: T,
    frame_buffer_view: Option<wgpu::TextureView>,
    depth_texture_view: wgpu::TextureView,
    copy_srgb_view: Option<wgpu::TextureView>,
    copy_srgb_bind_group: Option<wgpu::BindGroup>,
    current_frame: Option<Frame<'static, T>>,
    meshes: Vec<Mesh>,
    mask_state: MaskState,
    shape_tessellator: ShapeTessellator,
    num_masks: u32,
    quad_vbo: wgpu::Buffer,
    quad_ibo: wgpu::Buffer,
    quad_tex_transforms: wgpu::Buffer,
    blend_modes: Vec<BlendMode>,
    bitmap_registry: FnvHashMap<BitmapHandle, RegistryData>,
    next_bitmap_handle: BitmapHandle,
    // This is currently unused - we just store it to report in
    // `get_viewport_dimensions`
    viewport_scale_factor: f64,
    offscreen: bool,
}

impl WgpuRenderBackend<SwapChainTarget> {
    #[cfg(target_family = "wasm")]
    pub async fn for_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL);
        let surface = instance.create_surface_from_canvas(canvas);
        let descriptors = Self::build_descriptors(
            wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            instance,
            Some(&surface),
            wgpu::PowerPreference::HighPerformance,
            None,
        )
        .await?;
        let target = SwapChainTarget::new(
            surface,
            descriptors.onscreen.surface_format,
            (1, 1),
            &descriptors.device,
        );
        Self::new(Arc::new(descriptors), target)
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn for_window<W: raw_window_handle::HasRawWindowHandle>(
        window: &W,
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            log::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(window) };
        let descriptors = futures::executor::block_on(Self::build_descriptors(
            backend,
            instance,
            Some(&surface),
            power_preference,
            trace_path,
        ))?;
        let target = SwapChainTarget::new(
            surface,
            descriptors.onscreen.surface_format,
            size,
            &descriptors.device,
        );
        Self::new(Arc::new(descriptors), target)
    }
}

#[cfg(not(target_family = "wasm"))]
impl WgpuRenderBackend<target::TextureTarget> {
    pub fn for_offscreen(
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            log::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(backend);
        let descriptors = futures::executor::block_on(Self::build_descriptors(
            backend,
            instance,
            None,
            power_preference,
            trace_path,
        ))?;
        let target = target::TextureTarget::new(&descriptors.device, size)?;
        Self::new(Arc::new(descriptors), target)
    }

    pub fn capture_frame(&self, premultiplied_alpha: bool) -> Option<image::RgbaImage> {
        self.target
            .capture(&self.descriptors.device, premultiplied_alpha)
    }
}

macro_rules! target_data {
    ($this:expr) => {{
        if $this.offscreen {
            &$this.descriptors.offscreen
        } else {
            &$this.descriptors.onscreen
        }
    }};
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub fn new(descriptors: Arc<Descriptors>, target: T) -> Result<Self, Error> {
        if target.width() > descriptors.limits.max_texture_dimension_2d
            || target.height() > descriptors.limits.max_texture_dimension_2d
        {
            return Err(format!(
                "Render target texture cannot be larger than {}px on either dimension (requested {} x {})",
                descriptors.limits.max_texture_dimension_2d,
                target.width(),
                target.height()
            )
                .into());
        }

        let extent = wgpu::Extent3d {
            width: target.width(),
            height: target.height(),
            depth_or_array_layers: 1,
        };

        let frame_buffer_view = if descriptors.onscreen.msaa_sample_count > 1 {
            let frame_buffer = descriptors.device.create_texture(&wgpu::TextureDescriptor {
                label: create_debug_label!("Framebuffer texture").as_deref(),
                size: extent,
                mip_level_count: 1,
                sample_count: descriptors.onscreen.msaa_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: descriptors.onscreen.frame_buffer_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
            Some(frame_buffer.create_view(&Default::default()))
        } else {
            None
        };

        let depth_texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Depth texture").as_deref(),
            size: extent,
            mip_level_count: 1,
            sample_count: descriptors.onscreen.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_view(&Default::default());

        let (quad_vbo, quad_ibo, quad_tex_transforms) = create_quad_buffers(&descriptors.device);

        let (copy_srgb_view, copy_srgb_bind_group) = if descriptors.onscreen.frame_buffer_format
            != descriptors.offscreen.surface_format
        {
            let copy_srgb_buffer = descriptors.device.create_texture(&wgpu::TextureDescriptor {
                label: create_debug_label!("Copy sRGB framebuffer texture").as_deref(),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: descriptors.onscreen.frame_buffer_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            });
            let copy_srgb_view = copy_srgb_buffer.create_view(&Default::default());
            let copy_srgb_bind_group =
                descriptors
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &descriptors.onscreen.pipelines.bitmap_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &quad_tex_transforms,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<TextureTransforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&copy_srgb_view),
                            },
                        ],
                        label: create_debug_label!("Copy sRGB bind group").as_deref(),
                    });
            (Some(copy_srgb_view), Some(copy_srgb_bind_group))
        } else {
            (None, None)
        };

        let mut globals = Globals::new(&descriptors.device, &descriptors.globals_layout);
        globals.set_resolution(target.width(), target.height());

        let uniform_buffers =
            UniformBuffer::new(descriptors.limits.min_uniform_buffer_offset_alignment);

        Ok(Self {
            descriptors,
            globals,
            uniform_buffers,
            target,
            frame_buffer_view,
            depth_texture_view,
            copy_srgb_view,
            copy_srgb_bind_group,
            current_frame: None,
            meshes: Vec::new(),
            shape_tessellator: ShapeTessellator::new(),

            num_masks: 0,
            mask_state: MaskState::NoMask,

            quad_vbo,
            quad_ibo,
            quad_tex_transforms,
            blend_modes: vec![BlendMode::Normal],
            bitmap_registry: Default::default(),
            next_bitmap_handle: BitmapHandle(0),
            viewport_scale_factor: 1.0,
            offscreen: false,
        })
    }

    pub async fn build_descriptors(
        backend: wgpu::Backends,
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface>,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Descriptors, Error> {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface,
            force_fallback_adapter: false,
        }).await
            .ok_or_else(|| {
                let names = get_backend_names(backend);
                if names.is_empty() {
                    "Ruffle requires hardware acceleration, but no compatible graphics device was found (no backend provided?)".to_string()
                } else if cfg!(any(windows, target_os = "macos")) {
                    format!("Ruffle does not support OpenGL on {}.", if cfg!(windows) { "Windows" } else { "macOS" })
                } else {
                    format!("Ruffle requires hardware acceleration, but no compatible graphics device was found supporting {}", format_list(&names, "or"))
                }
            })?;

        let (device, queue) = request_device(&adapter, trace_path).await?;
        let info = adapter.get_info();
        // Ideally we want to use an RGBA non-sRGB surface format, because Flash colors and
        // blending are done in sRGB space -- we don't want the GPU to adjust the colors.
        // Some platforms may only support an sRGB surface, in which case we will draw to an
        // intermediate linear buffer and then copy to the sRGB surface.
        let surface_format = surface
            .and_then(|surface| {
                let formats = surface.get_supported_formats(&adapter);
                formats
                    .iter()
                    .find(|format| {
                        matches!(
                            format,
                            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                        )
                    })
                    .or_else(|| formats.first())
                    .cloned()
            })
            // No surface (rendering to texture), default to linear RBGA.
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);
        // TODO: Allow the sample count to be set from command line/settings file.
        Ok(Descriptors::new(
            device,
            queue,
            info,
            surface_format,
            DEFAULT_SAMPLE_COUNT,
        ))
    }

    fn register_shape_internal(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> Mesh {
        let shape_id = shape.id; // TODO: remove?
        let lyon_mesh = self
            .shape_tessellator
            .tessellate_shape(shape, bitmap_source);

        let mut draws = Vec::with_capacity(lyon_mesh.len());
        for draw in lyon_mesh {
            let vertices: Vec<_> = draw.vertices.into_iter().map(Vertex::from).collect();
            let vertex_buffer = create_buffer_with_data(
                &self.descriptors.device,
                bytemuck::cast_slice(&vertices),
                wgpu::BufferUsages::VERTEX,
                create_debug_label!("Shape {} ({}) vbo", shape_id, draw.draw_type.name()),
            );

            let index_buffer = create_buffer_with_data(
                &self.descriptors.device,
                bytemuck::cast_slice(&draw.indices),
                wgpu::BufferUsages::INDEX,
                create_debug_label!("Shape {} ({}) ibo", shape_id, draw.draw_type.name()),
            );

            let index_count = draw.indices.len() as u32;
            let draw_id = draws.len();

            draws.push(match draw.draw_type {
                TessDrawType::Color => Draw {
                    draw_type: DrawType::Color,
                    vertex_buffer,
                    index_buffer,
                    num_indices: index_count,
                    num_mask_indices: draw.mask_index_count,
                },
                TessDrawType::Gradient(gradient) => {
                    // TODO: Extract to function?
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&gradient.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&gradient.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&gradient.matrix[2]);

                    let tex_transforms_ubo = create_buffer_with_data(
                        &self.descriptors.device,
                        bytemuck::cast_slice(&[texture_transform]),
                        wgpu::BufferUsages::UNIFORM,
                        create_debug_label!(
                            "Shape {} draw {} textransforms ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    );

                    let (gradient_ubo, buffer_size) = if self
                        .descriptors
                        .limits
                        .max_storage_buffers_per_shader_stage
                        > 0
                    {
                        (
                            create_buffer_with_data(
                                &self.descriptors.device,
                                bytemuck::cast_slice(&[GradientStorage::from(gradient)]),
                                wgpu::BufferUsages::STORAGE,
                                create_debug_label!(
                                    "Shape {} draw {} gradient ubo transfer buffer",
                                    shape_id,
                                    draw_id
                                ),
                            ),
                            wgpu::BufferSize::new(std::mem::size_of::<GradientStorage>() as u64),
                        )
                    } else {
                        (
                            create_buffer_with_data(
                                &self.descriptors.device,
                                bytemuck::cast_slice(&[GradientUniforms::from(gradient)]),
                                wgpu::BufferUsages::UNIFORM,
                                create_debug_label!(
                                    "Shape {} draw {} gradient ubo transfer buffer",
                                    shape_id,
                                    draw_id
                                ),
                            ),
                            wgpu::BufferSize::new(std::mem::size_of::<GradientUniforms>() as u64),
                        )
                    };

                    let bind_group_label = create_debug_label!(
                        "Shape {} (gradient) draw {} bindgroup",
                        shape_id,
                        draw_id
                    );
                    let bind_group =
                        self.descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &target_data!(self).pipelines.gradient_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &tex_transforms_ubo,
                                                offset: 0,
                                                size: wgpu::BufferSize::new(std::mem::size_of::<
                                                    TextureTransforms,
                                                >(
                                                )
                                                    as u64),
                                            },
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &gradient_ubo,
                                                offset: 0,
                                                size: buffer_size,
                                            },
                                        ),
                                    },
                                ],
                                label: bind_group_label.as_deref(),
                            });

                    Draw {
                        draw_type: DrawType::Gradient {
                            texture_transforms: tex_transforms_ubo,
                            gradient: gradient_ubo,
                            bind_group,
                        },
                        vertex_buffer,
                        index_buffer,
                        num_indices: index_count,
                        num_mask_indices: draw.mask_index_count,
                    }
                }
                TessDrawType::Bitmap(bitmap) => {
                    let entry = self.bitmap_registry.get(&bitmap.bitmap).unwrap();
                    let texture_view = entry
                        .texture_wrapper
                        .texture
                        .create_view(&Default::default());

                    // TODO: Extract to function?
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&bitmap.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&bitmap.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&bitmap.matrix[2]);

                    let tex_transforms_ubo = create_buffer_with_data(
                        &self.descriptors.device,
                        bytemuck::cast_slice(&[texture_transform]),
                        wgpu::BufferUsages::UNIFORM,
                        create_debug_label!(
                            "Shape {} draw {} textransforms ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    );

                    let bind_group_label = create_debug_label!(
                        "Shape {} (bitmap) draw {} bindgroup",
                        shape_id,
                        draw_id
                    );
                    let bind_group =
                        self.descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &target_data!(self).pipelines.bitmap_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &tex_transforms_ubo,
                                                offset: 0,
                                                size: wgpu::BufferSize::new(std::mem::size_of::<
                                                    TextureTransforms,
                                                >(
                                                )
                                                    as u64),
                                            },
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&texture_view),
                                    },
                                ],
                                label: bind_group_label.as_deref(),
                            });

                    Draw {
                        draw_type: DrawType::Bitmap {
                            texture_transforms: tex_transforms_ubo,
                            texture_view,
                            is_smoothed: bitmap.is_smoothed,
                            is_repeating: bitmap.is_repeating,
                            bind_group,
                        },
                        vertex_buffer,
                        index_buffer,
                        num_indices: index_count,
                        num_mask_indices: draw.mask_index_count,
                    }
                }
            });
        }

        Mesh { draws }
    }

    fn blend_mode(&self) -> BlendMode {
        *self.blend_modes.last().unwrap()
    }

    pub fn descriptors(&self) -> &Arc<Descriptors> {
        &self.descriptors
    }

    fn begin_frame(&mut self, clear: Color) {
        self.mask_state = MaskState::NoMask;
        self.num_masks = 0;
        self.uniform_buffers.reset();

        let frame_output = match self.target.get_next_texture() {
            Ok(frame) => frame,
            Err(e) => {
                log::warn!("Couldn't begin new render frame: {}", e);
                // Attemp to recreate the swap chain in this case.
                self.target.resize(
                    &self.descriptors.device,
                    self.target.width(),
                    self.target.height(),
                );
                return;
            }
        };

        let label = create_debug_label!("Draw encoder");
        let draw_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });
        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let uniform_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });
        let mut frame_data = Box::new((draw_encoder, frame_output, uniform_encoder));

        self.globals
            .update_uniform(&self.descriptors.device, &mut frame_data.0);

        // Use intermediate render targets when resolving MSAA or copying from linear-to-sRGB texture.
        let (color_view, resolve_target) = match (&self.frame_buffer_view, &self.copy_srgb_view) {
            (None, None) => (frame_data.1.view(), None),
            (None, Some(copy)) => (copy, None),
            (Some(frame_buffer), None) => (frame_buffer, Some(frame_data.1.view())),
            (Some(frame_buffer), Some(copy)) => (frame_buffer, Some(copy)),
        };

        let render_pass = frame_data.0.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(clear.r) / 255.0,
                        g: f64::from(clear.g) / 255.0,
                        b: f64::from(clear.b) / 255.0,
                        a: f64::from(clear.a) / 255.0,
                    }),
                    store: true,
                },
                resolve_target,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: false,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
            label: None,
        });

        // Since RenderPass holds a reference to the CommandEncoder, we cast the lifetime
        // away to allow for the self-referencing struct. draw_encoder is boxed so its
        // address should remain stable.
        self.current_frame = Some(Frame {
            render_pass: unsafe {
                std::mem::transmute::<_, wgpu::RenderPass<'static>>(render_pass)
            },
            frame_data,
        });
    }

    fn end_frame(&mut self) {
        if let Some(frame) = self.current_frame.take() {
            let draw_encoder = frame.frame_data.0;
            let mut uniform_encoder = frame.frame_data.2;
            let render_pass = frame.render_pass;
            // Finalize render pass.
            drop(render_pass);

            // If we have an sRGB surface, copy from our linear intermediate buffer to the sRGB surface.
            let command_buffers = if let Some(copy_srgb_bind_group) = &self.copy_srgb_bind_group {
                debug_assert!(self.copy_srgb_view.is_some());
                let mut copy_encoder = self.descriptors.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: create_debug_label!("Frame copy command encoder").as_deref(),
                    },
                );

                let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: frame.frame_data.1.view(),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: true,
                        },
                        resolve_target: None,
                    })],
                    depth_stencil_attachment: None,
                    label: None,
                });

                render_pass.set_pipeline(&self.descriptors.pipelines.copy_srgb_pipeline);
                render_pass.set_bind_group(0, self.globals.bind_group(), &[]);
                self.uniform_buffers.write_uniforms(
                    &self.descriptors.device,
                    &self.descriptors.uniform_buffers_layout,
                    &mut uniform_encoder,
                    &mut render_pass,
                    1,
                    &Transforms {
                        world_matrix: [
                            [self.target.width() as f32, 0.0, 0.0, 0.0],
                            [0.0, self.target.height() as f32, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ],
                        color_adjustments: ColorAdjustments {
                            mult_color: [1.0, 1.0, 1.0, 1.0],
                            add_color: [0.0, 0.0, 0.0, 0.0],
                        },
                    },
                );
                render_pass.set_bind_group(2, copy_srgb_bind_group, &[]);
                render_pass.set_bind_group(
                    3,
                    self.descriptors
                        .bitmap_samplers
                        .get_bind_group(false, false),
                    &[],
                );
                render_pass.set_vertex_buffer(0, self.quad_vbo.slice(..));
                render_pass.set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..6, 0, 0..1);
                drop(render_pass);
                vec![
                    uniform_encoder.finish(),
                    draw_encoder.finish(),
                    copy_encoder.finish(),
                ]
            } else {
                vec![uniform_encoder.finish(), draw_encoder.finish()]
            };

            self.uniform_buffers.finish();
            self.target.submit(
                &self.descriptors.device,
                &self.descriptors.queue,
                command_buffers,
                frame.frame_data.1,
            );
        }
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.descriptors.device
    }

    fn render_offscreen_internal(
        mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        commands: CommandList,
        clear_color: Color,
    ) -> Result<(Self, Bitmap), ruffle_render::error::Error> {
        // We need ownership of `Texture` to access the non-`Clone`
        // `wgpu` fields. At the end of this method, we re-insert
        // `texture` into the map.
        //
        // This means that the target texture will be inaccessible
        // while the callback `f` is a problem. This would only be
        // an issue if a caller tried to render the target texture
        // to itself, which probably isn't supported by Flash. If it
        // is, then we could change `TextureTarget` to use an `Rc<wgpu::Texture>`
        let mut texture = self.bitmap_registry.remove(&handle).unwrap();

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        if self.offscreen {
            panic!("Nested render_onto_bitmap is not supported!")
        }

        let descriptors = self.descriptors.clone();

        // We will (presumably) never render to the majority of textures, so
        // we lazily create the buffer and depth texture.
        // Once created, we never destroy this data, under the assumption
        // that the SWF will try to render to this more than once.
        //
        // If we end up hitting wgpu device limits due to having too
        // many buffers / depth textures rendered at once, we could
        // try storing this data in an LRU cache, evicting entries
        // as needed.
        let mut texture_offscreen =
            texture
                .texture_wrapper
                .texture_offscreen
                .unwrap_or_else(|| {
                    let depth_texture_view =
                        create_depth_texture_view(&descriptors, &descriptors.offscreen, extent);
                    let buffer_dimensions = BufferDimensions::new(width as usize, height as usize);
                    let buffer_label = create_debug_label!("Render target buffer");
                    let buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
                        label: buffer_label.as_deref(),
                        size: (buffer_dimensions.padded_bytes_per_row.get() as u64
                            * buffer_dimensions.height as u64),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    });
                    TextureOffscreen {
                        depth_texture_view,
                        buffer,
                        buffer_dimensions,
                    }
                });

        let target = TextureTarget {
            size: extent,
            texture: texture.texture_wrapper.texture,
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: texture_offscreen.buffer,
            buffer_dimensions: texture_offscreen.buffer_dimensions,
        };

        let (old_width, old_height) = self.globals.resolution();

        // Is it worth caching this?
        self.globals.set_resolution(width, height);

        let mut texture_backend = WgpuRenderBackend {
            descriptors,
            target,
            // FIXME - Enable MSAA for textures
            frame_buffer_view: None,
            depth_texture_view: texture_offscreen.depth_texture_view,
            // We explicitly request a non-SRGB format for textures
            copy_srgb_view: None,
            copy_srgb_bind_group: None,
            current_frame: None,
            meshes: self.meshes,
            mask_state: MaskState::NoMask,
            shape_tessellator: self.shape_tessellator,
            num_masks: 0,
            quad_vbo: self.quad_vbo,
            quad_ibo: self.quad_ibo,
            quad_tex_transforms: self.quad_tex_transforms,
            bitmap_registry: self.bitmap_registry,
            offscreen: true,
            next_bitmap_handle: self.next_bitmap_handle,
            globals: self.globals,
            uniform_buffers: self.uniform_buffers,
            viewport_scale_factor: self.viewport_scale_factor,
            blend_modes: vec![BlendMode::Normal],
        };

        texture_backend.submit_frame(clear_color, commands);

        // Capture with premultiplied alpha, which is what we use for all textures
        let image = texture_backend
            .target
            .capture(&texture_backend.descriptors.device, true);

        let image = image.map(|image| {
            Bitmap::new(
                image.dimensions().0,
                image.dimensions().1,
                ruffle_render::bitmap::BitmapFormat::Rgba,
                image.into_raw(),
            )
        });

        self.offscreen = false;
        self.meshes = texture_backend.meshes;
        self.shape_tessellator = texture_backend.shape_tessellator;
        self.bitmap_registry = texture_backend.bitmap_registry;
        self.quad_tex_transforms = texture_backend.quad_tex_transforms;
        self.quad_ibo = texture_backend.quad_ibo;
        self.quad_vbo = texture_backend.quad_vbo;
        self.globals = texture_backend.globals;
        self.uniform_buffers = texture_backend.uniform_buffers;
        self.next_bitmap_handle = texture_backend.next_bitmap_handle;
        self.globals.set_resolution(old_width, old_height);

        texture_offscreen.buffer = texture_backend.target.buffer;
        texture_offscreen.buffer_dimensions = texture_backend.target.buffer_dimensions;
        texture_offscreen.depth_texture_view = texture_backend.depth_texture_view;
        texture.texture_wrapper.texture_offscreen = Some(texture_offscreen);
        texture.texture_wrapper.texture = texture_backend.target.texture;
        self.bitmap_registry.insert(handle, texture);

        Ok((self, image.unwrap()))
    }
}

impl<T: RenderTarget + 'static> RenderBackend for WgpuRenderBackend<T> {
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        // Avoid panics from creating 0-sized framebuffers.
        // TODO: find a way to bubble an error when the size is too large
        let width = std::cmp::max(
            std::cmp::min(
                dimensions.width,
                self.descriptors.limits.max_texture_dimension_2d,
            ),
            1,
        );
        let height = std::cmp::max(
            std::cmp::min(
                dimensions.height,
                self.descriptors.limits.max_texture_dimension_2d,
            ),
            1,
        );
        self.target.resize(&self.descriptors.device, width, height);

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.frame_buffer_view = if target_data!(self).msaa_sample_count > 1 {
            let frame_buffer = self
                .descriptors
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: create_debug_label!("Framebuffer texture").as_deref(),
                    size,
                    mip_level_count: 1,
                    sample_count: target_data!(self).msaa_sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: target_data!(self).frame_buffer_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                });
            Some(frame_buffer.create_view(&Default::default()))
        } else {
            None
        };

        let depth_texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: create_debug_label!("Depth texture").as_deref(),
                size,
                mip_level_count: 1,
                sample_count: target_data!(self).msaa_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
        self.depth_texture_view = depth_texture.create_view(&Default::default());

        (self.copy_srgb_view, self.copy_srgb_bind_group) = if target_data!(self).frame_buffer_format
            != target_data!(self).surface_format
        {
            let copy_srgb_buffer =
                self.descriptors
                    .device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: create_debug_label!("Copy sRGB framebuffer texture").as_deref(),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: target_data!(self).frame_buffer_format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                    });
            let copy_srgb_view = copy_srgb_buffer.create_view(&Default::default());
            let copy_srgb_bind_group =
                self.descriptors
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &target_data!(self).pipelines.bitmap_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.quad_tex_transforms,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<TextureTransforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&copy_srgb_view),
                            },
                        ],
                        label: create_debug_label!("Copy sRGB bind group").as_deref(),
                    });
            (Some(copy_srgb_view), Some(copy_srgb_bind_group))
        } else {
            (None, None)
        };

        self.globals.set_resolution(width, height);
        self.viewport_scale_factor = dimensions.scale_factor;
    }

    fn viewport_dimensions(&self) -> ViewportDimensions {
        ViewportDimensions {
            width: self.target.width(),
            height: self.target.height(),
            scale_factor: self.viewport_scale_factor,
        }
    }

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes.push(mesh);
        handle
    }

    fn replace_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        handle: ShapeHandle,
    ) {
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes[handle.0] = mesh;
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = ruffle_render::shape_utils::swf_glyph_to_shape(glyph);
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(
            (&shape).into(),
            &ruffle_render::backend::null::NullBitmapSource,
        );
        self.meshes.push(mesh);
        handle
    }

    fn submit_frame(&mut self, clear: Color, commands: CommandList) {
        self.begin_frame(clear);
        commands.execute(self);
        self.end_frame();
    }

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap> {
        self.bitmap_registry.get(&bitmap).map(|e| e.bitmap.clone())
    }

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, BitmapError> {
        if bitmap.width() > self.descriptors.limits.max_texture_dimension_2d
            || bitmap.height() > self.descriptors.limits.max_texture_dimension_2d
        {
            return Err(BitmapError::TooLarge);
        }

        let bitmap = bitmap.to_rgba();
        let extent = wgpu::Extent3d {
            width: bitmap.width(),
            height: bitmap.height(),
            depth_or_array_layers: 1,
        };

        let texture_label = create_debug_label!("Bitmap");
        let texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: texture_label.as_deref(),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
            });

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            bitmap.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        let handle = self.next_bitmap_handle;
        self.next_bitmap_handle = BitmapHandle(self.next_bitmap_handle.0 + 1);
        let width = bitmap.width();
        let height = bitmap.height();

        // Make bind group for bitmap quad.
        let texture_view = texture.create_view(&Default::default());
        let bind_group = self
            .descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &target_data!(self).pipelines.bitmap_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.quad_tex_transforms,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                ],
                label: create_debug_label!("Bitmap {} bind group", handle.0).as_deref(),
            });

        if self
            .bitmap_registry
            .insert(
                handle,
                RegistryData {
                    bitmap,
                    texture_wrapper: Texture {
                        width,
                        height,
                        texture,
                        bind_group,
                        texture_offscreen: None,
                    },
                },
            )
            .is_some()
        {
            panic!("Overwrote existing bitmap {:?}", handle);
        }

        Ok(handle)
    }

    fn unregister_bitmap(&mut self, handle: BitmapHandle) {
        self.bitmap_registry.remove(&handle);
    }

    fn update_texture(
        &mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, BitmapError> {
        let texture = if let Some(entry) = self.bitmap_registry.get(&handle) {
            &entry.texture_wrapper.texture
        } else {
            log::warn!("Tried to replace nonexistent texture");
            return Ok(handle);
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        Ok(handle)
    }

    fn render_offscreen(
        &mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        commands: CommandList,
        clear_color: Color,
    ) -> Result<Bitmap, ruffle_render::error::Error> {
        // Rendering to a texture backend requires us to use non-`Clone`
        // wgpu resources (e.g. `wgpu::Device`, `wgpu::Queue`.
        //
        // We expect that in the majority of SWFs, we will spend most
        // of our time performing 'normal' (non-offscreen) renders.
        // Therefore, we want to avoid penalizing this case by adding
        // in a check for 'normal' or 'offscreen' mode in the main
        // rendering code.
        //
        // To accomplish this, we use `take_mut` to temporarily
        // move out of `self`. This allows us to construct a new
        // `WgpuRenderBackend` with a `TextureTarget` corresponding to
        // `handle`. This allows us to re-use many of the fields from
        // our normal `WgpuRenderBackend` without wrapping in an `Rc`
        // or other indirection.
        //
        // Note that `take_mut` causes the process to abort if the
        // `with_offscreen_render_backend_internal` panics, since
        // the `&mut self` reference would be logically uninitialized.
        // However, we normally compile Ruffle with `panic=abort`,
        // so this shouldn't actually have an effect in practice.
        // Even with `panic=unwind`, we would still get a backtrace
        // printed, and there's not really much point in attempting
        // to recover from a partially failed render operation, anyway.
        Ok(take_mut(self, |this| {
            this.render_offscreen_internal(handle, width, height, commands, clear_color)
                .expect("Failed to render to offscreen backend")
        }))
    }
}

impl<T: RenderTarget> CommandHandler for WgpuRenderBackend<T> {
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        let target_data = target_data!(self);
        if let Some(entry) = self.bitmap_registry.get(&bitmap) {
            let texture = &entry.texture_wrapper;
            let blend_mode = self.blend_mode();
            let frame = if let Some(frame) = &mut self.current_frame {
                frame.get()
            } else {
                return;
            };

            let transform = Transform {
                matrix: transform.matrix
                    * ruffle_render::matrix::Matrix {
                        a: texture.width as f32,
                        d: texture.height as f32,
                        ..Default::default()
                    },
                ..*transform
            };

            let world_matrix = [
                [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
                [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    transform.matrix.tx.to_pixels() as f32,
                    transform.matrix.ty.to_pixels() as f32,
                    0.0,
                    1.0,
                ],
            ];

            frame.render_pass.set_pipeline(
                target_data
                    .pipelines
                    .bitmap_pipelines
                    .pipeline_for(blend_mode.into(), self.mask_state),
            );
            frame
                .render_pass
                .set_bind_group(0, self.globals.bind_group(), &[]);

            self.uniform_buffers.write_uniforms(
                &self.descriptors.device,
                &self.descriptors.uniform_buffers_layout,
                &mut frame.frame_data.2,
                &mut frame.render_pass,
                1,
                &Transforms {
                    world_matrix,
                    color_adjustments: ColorAdjustments::from(transform.color_transform),
                },
            );

            frame
                .render_pass
                .set_bind_group(2, &texture.bind_group, &[]);
            frame.render_pass.set_bind_group(
                3,
                self.descriptors
                    .bitmap_samplers
                    .get_bind_group(false, smoothing),
                &[],
            );
            frame
                .render_pass
                .set_vertex_buffer(0, self.quad_vbo.slice(..));
            frame
                .render_pass
                .set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);

            match self.mask_state {
                MaskState::NoMask => (),
                MaskState::DrawMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks);
                }
            };

            frame.render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let blend_mode = self.blend_mode();
        let frame = if let Some(frame) = &mut self.current_frame {
            frame.get()
        } else {
            return;
        };

        let mesh = &mut self.meshes[shape.0];

        let world_matrix = [
            [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
            [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                transform.matrix.tx.to_pixels() as f32,
                transform.matrix.ty.to_pixels() as f32,
                0.0,
                1.0,
            ],
        ];

        frame
            .render_pass
            .set_bind_group(0, self.globals.bind_group(), &[]);

        self.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &self.descriptors.uniform_buffers_layout,
            &mut frame.frame_data.2,
            &mut frame.render_pass,
            1,
            &Transforms {
                world_matrix,
                color_adjustments: ColorAdjustments::from(transform.color_transform),
            },
        );

        for draw in &mesh.draws {
            let num_indices = if self.mask_state != MaskState::DrawMaskStencil
                && self.mask_state != MaskState::ClearMaskStencil
            {
                draw.num_indices
            } else {
                // Omit strokes when drawing a mask stencil.
                draw.num_mask_indices
            };
            if num_indices == 0 {
                continue;
            }

            match &draw.draw_type {
                DrawType::Color => {
                    frame.render_pass.set_pipeline(
                        target_data!(self)
                            .pipelines
                            .color_pipelines
                            .pipeline_for(blend_mode.into(), self.mask_state),
                    );
                }
                DrawType::Gradient { bind_group, .. } => {
                    frame.render_pass.set_pipeline(
                        target_data!(self)
                            .pipelines
                            .gradient_pipelines
                            .pipeline_for(blend_mode.into(), self.mask_state),
                    );
                    frame.render_pass.set_bind_group(2, bind_group, &[]);
                }
                DrawType::Bitmap {
                    is_repeating,
                    is_smoothed,
                    bind_group,
                    ..
                } => {
                    frame.render_pass.set_pipeline(
                        target_data!(self)
                            .pipelines
                            .bitmap_pipelines
                            .pipeline_for(blend_mode.into(), self.mask_state),
                    );
                    frame.render_pass.set_bind_group(2, bind_group, &[]);
                    frame.render_pass.set_bind_group(
                        3,
                        self.descriptors
                            .bitmap_samplers
                            .get_bind_group(*is_repeating, *is_smoothed),
                        &[],
                    );
                }
            }

            frame
                .render_pass
                .set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            frame
                .render_pass
                .set_index_buffer(draw.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            match self.mask_state {
                MaskState::NoMask => (),
                MaskState::DrawMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks);
                }
            };

            frame.render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &ruffle_render::matrix::Matrix) {
        let blend_mode = self.blend_mode();
        let frame = if let Some(frame) = &mut self.current_frame {
            frame.get()
        } else {
            return;
        };

        let world_matrix = [
            [matrix.a, matrix.b, 0.0, 0.0],
            [matrix.c, matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                matrix.tx.to_pixels() as f32,
                matrix.ty.to_pixels() as f32,
                0.0,
                1.0,
            ],
        ];

        let mult_color = [
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        ];

        let add_color = [0.0, 0.0, 0.0, 0.0];
        frame.render_pass.set_pipeline(
            target_data!(self)
                .pipelines
                .color_pipelines
                .pipeline_for(blend_mode.into(), self.mask_state),
        );

        frame
            .render_pass
            .set_bind_group(0, self.globals.bind_group(), &[]);

        self.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &self.descriptors.uniform_buffers_layout,
            &mut frame.frame_data.2,
            &mut frame.render_pass,
            1,
            &Transforms {
                world_matrix,
                color_adjustments: ColorAdjustments {
                    mult_color,
                    add_color,
                },
            },
        );

        frame
            .render_pass
            .set_vertex_buffer(0, self.quad_vbo.slice(..));
        frame
            .render_pass
            .set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);

        match self.mask_state {
            MaskState::NoMask => (),
            MaskState::DrawMaskStencil => {
                debug_assert!(self.num_masks > 0);
                frame.render_pass.set_stencil_reference(self.num_masks - 1);
            }
            MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                debug_assert!(self.num_masks > 0);
                frame.render_pass.set_stencil_reference(self.num_masks);
            }
        };

        frame.render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn push_mask(&mut self) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
    }

    fn activate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
    }

    fn deactivate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
    }

    fn pop_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        self.mask_state = if self.num_masks == 0 {
            MaskState::NoMask
        } else {
            MaskState::DrawMaskedContent
        };
    }

    fn push_blend_mode(&mut self, blend: BlendMode) {
        self.blend_modes.push(blend);
    }

    fn pop_blend_mode(&mut self) {
        self.blend_modes.pop();
    }
}

fn create_depth_texture_view(
    descriptors: &Descriptors,
    target_data: &DescriptorsTargetData,
    extent: wgpu::Extent3d,
) -> wgpu::TextureView {
    let depth_texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
        label: create_debug_label!("Depth texture").as_deref(),
        size: extent,
        mip_level_count: 1,
        sample_count: target_data.msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    });
    depth_texture.create_view(&Default::default())
}

// Based on https://github.com/Sgeo/take_mut
fn take_mut<T, R, F>(mut_ref: &mut T, closure: F) -> R
where
    F: FnOnce(T) -> (T, R),
{
    unsafe {
        let old_t = std::ptr::read(mut_ref);
        let (new_t, ret) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure(old_t)))
                .unwrap_or_else(|e| {
                    eprintln!("Caught panic: {:?}", e);
                    ::std::process::abort()
                });
        std::ptr::write(mut_ref, new_t);
        ret
    }
}

// We try to request the highest limits we can get away with
async fn request_device(
    adapter: &wgpu::Adapter,
    trace_path: Option<&Path>,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    // We start off with the lowest limits we actually need - basically GL-ES 3.0
    let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
    // Then we increase parts of it to the maximum supported by the adapter, to take advantage of
    // more powerful hardware or capabilities
    limits = limits.using_resolution(adapter.limits());
    limits = limits.using_alignment(adapter.limits());

    limits.max_storage_buffers_per_shader_stage =
        adapter.limits().max_storage_buffers_per_shader_stage;
    limits.max_storage_buffer_binding_size = adapter.limits().max_storage_buffer_binding_size;

    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits,
            },
            trace_path,
        )
        .await
}
