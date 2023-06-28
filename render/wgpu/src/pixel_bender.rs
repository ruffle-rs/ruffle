use std::cell::RefCell;
use std::num::NonZeroU64;
use std::{borrow::Cow, cell::Cell, sync::Arc};

use indexmap::IndexMap;
use ruffle_render::error::Error as BitmapError;
use ruffle_render::pixel_bender::{
    PixelBenderShaderHandle, PixelBenderShaderImpl, PixelBenderType, OUT_COORD_NAME,
};
use ruffle_render::{
    bitmap::{BitmapHandle, PixelRegion, SyncHandle},
    pixel_bender::{PixelBenderParam, PixelBenderShader, PixelBenderShaderArgument},
};
use wgpu::util::StagingBelt;
use wgpu::{
    BindGroupEntry, BindingResource, BlendComponent, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, FrontFace, ImageCopyTexture, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModuleDescriptor, TextureDescriptor, TextureFormat, TextureView,
    VertexState,
};

use crate::{
    as_texture,
    backend::WgpuRenderBackend,
    descriptors::Descriptors,
    pipelines::VERTEX_BUFFERS_DESCRIPTION_POS,
    target::{RenderTarget, RenderTargetFrame, TextureTarget},
    QueueSyncHandle, Texture,
};

#[derive(Debug)]
pub struct PixelBenderWgpuShader {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    shader: PixelBenderShader,
    float_parameters_buffer: wgpu::Buffer,
    float_parameters_buffer_size: u64,
    int_parameters_buffer: wgpu::Buffer,
    int_parameters_buffer_size: u64,
    staging_belt: RefCell<StagingBelt>,
}

impl PixelBenderShaderImpl for PixelBenderWgpuShader {
    fn parsed_shader(&self) -> &PixelBenderShader {
        &self.shader
    }
}

pub fn as_cache_holder(handle: &PixelBenderShaderHandle) -> &PixelBenderWgpuShader {
    <dyn PixelBenderShaderImpl>::downcast_ref(&*handle.0).unwrap()
}

impl PixelBenderWgpuShader {
    pub fn new(descriptors: &Descriptors, shader: PixelBenderShader) -> PixelBenderWgpuShader {
        let mut layout_entries = vec![
            // One sampler per filter/wrapping combination - see BitmapFilters
            // An AGAL shader can use any of these samplers, so
            // we need to bind them all.
            wgpu::BindGroupLayoutEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_NEAREST,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_LINEAR,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_BILINEAR,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: naga_pixelbender::SHADER_FLOAT_PARAMETERS_INDEX,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: naga_pixelbender::SHADER_INT_PARAMETERS_INDEX,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];

        for param in &shader.params {
            if let PixelBenderParam::Texture { index, .. } = param {
                let binding = naga_pixelbender::TEXTURE_START_BIND_INDEX + *index as u32;
                layout_entries.push(wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                });
            }
        }

        let globals_layout_label =
            create_debug_label!("PixelBender bind group layout for {:?}", shader.name);
        let bind_group_layout =
            descriptors
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: globals_layout_label.as_deref(),
                    entries: &layout_entries,
                });

        let pipeline_layout_label =
            create_debug_label!("PixelBender pipeline layout for {:?}", shader.name);
        let pipeline_layout =
            descriptors
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: pipeline_layout_label.as_deref(),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let shaders =
            naga_pixelbender::ShaderBuilder::build(&shader).expect("Failed to compile shader");

        let float_label =
            create_debug_label!("PixelBender float parameters buffer for {:?}", shader.name);

        let float_parameters_buffer = descriptors.device.create_buffer(&BufferDescriptor {
            label: float_label.as_deref(),
            size: shaders.float_parameters_buffer_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let int_label =
            create_debug_label!("PixelBender int parameters buffer for {:?}", shader.name);

        let int_parameters_buffer = descriptors.device.create_buffer(&BufferDescriptor {
            label: int_label.as_deref(),
            size: shaders.int_parameters_buffer_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_shader = descriptors
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(Cow::Owned(shaders.vertex)),
            });

        let fragment_shader = descriptors
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(Cow::Owned(shaders.fragment)),
            });

        let pipeline = descriptors
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: create_debug_label!("RenderPipeline").as_deref(),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &vertex_shader,
                    entry_point: naga_pixelbender::SHADER_ENTRYPOINT,
                    buffers: &VERTEX_BUFFERS_DESCRIPTION_POS,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: naga_pixelbender::SHADER_ENTRYPOINT,
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        // FIXME - what should this be?
                        blend: Some(wgpu::BlendState {
                            color: BlendComponent::OVER,
                            alpha: BlendComponent::OVER,
                        }),
                        write_mask: ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: Default::default(),
            });

        PixelBenderWgpuShader {
            bind_group_layout,
            pipeline,
            shader,
            float_parameters_buffer,
            float_parameters_buffer_size: shaders.float_parameters_buffer_size,
            int_parameters_buffer,
            int_parameters_buffer_size: shaders.int_parameters_buffer_size,
            // FIXME - come up with a good chunk size
            staging_belt: RefCell::new(StagingBelt::new(8)),
        }
    }
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub(super) fn compile_pixelbender_shader_impl(
        &mut self,
        shader: PixelBenderShader,
    ) -> Result<PixelBenderShaderHandle, BitmapError> {
        let handle = PixelBenderWgpuShader::new(&self.descriptors, shader);
        Ok(PixelBenderShaderHandle(Arc::new(handle)))
    }

    pub(super) fn run_pixelbender_shader_impl(
        &mut self,
        shader: PixelBenderShaderHandle,
        arguments: &[PixelBenderShaderArgument],
        target_handle: BitmapHandle,
    ) -> Result<Box<dyn SyncHandle>, BitmapError> {
        let compiled_shader = &as_cache_holder(&shader);
        let mut staging_belt = compiled_shader.staging_belt.borrow_mut();

        let mut arguments = arguments.to_vec();

        let target = as_texture(&target_handle);
        let extent = wgpu::Extent3d {
            width: target.texture.width(),
            height: target.texture.height(),
            depth_or_array_layers: 1,
        };

        let mut texture_target = TextureTarget {
            size: extent,
            texture: target.texture.clone(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: None,
        };

        let frame_output = texture_target
            .get_next_texture()
            .expect("TextureTargetFrame.get_next_texture is infallible");

        let mut bind_group_entries = vec![
            BindGroupEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_NEAREST,
                resource: BindingResource::Sampler(&self.descriptors.bitmap_samplers.clamp_nearest),
            },
            BindGroupEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_LINEAR,
                resource: BindingResource::Sampler(&self.descriptors.bitmap_samplers.clamp_linear),
            },
            BindGroupEntry {
                binding: naga_pixelbender::SAMPLER_CLAMP_BILINEAR,
                // FIXME - create bilinear sampler
                resource: BindingResource::Sampler(&self.descriptors.bitmap_samplers.clamp_linear),
            },
            BindGroupEntry {
                binding: naga_pixelbender::SHADER_FLOAT_PARAMETERS_INDEX,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &compiled_shader.float_parameters_buffer,
                    offset: 0,
                    size: Some(
                        NonZeroU64::new(compiled_shader.float_parameters_buffer_size).unwrap(),
                    ),
                }),
            },
            BindGroupEntry {
                binding: naga_pixelbender::SHADER_INT_PARAMETERS_INDEX,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &compiled_shader.int_parameters_buffer,
                    offset: 0,
                    size: Some(
                        NonZeroU64::new(compiled_shader.int_parameters_buffer_size).unwrap(),
                    ),
                }),
            },
        ];

        let mut texture_views: IndexMap<u8, TextureView> = Default::default();

        let mut render_command_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Render command encoder").as_deref(),
                });

        let mut target_clone = None;

        let mut float_offset = 0;
        let mut int_offset = 0;

        for input in &mut arguments {
            match input {
                PixelBenderShaderArgument::ImageInput { index, texture, .. } => {
                    // The input is the same as the output - we need to clone the input.
                    // We will write to the original output, and use a clone of the input as a texture input binding
                    if std::ptr::eq(
                        Arc::as_ptr(&texture.0) as *const (),
                        Arc::as_ptr(&target_handle.0) as *const (),
                    ) {
                        let cached_fresh_handle = target_clone.get_or_insert_with(|| {
                            let extent = wgpu::Extent3d {
                                width: target.texture.width(),
                                height: target.texture.height(),
                                depth_or_array_layers: 1,
                            };
                            let fresh_texture =
                                self.descriptors.device.create_texture(&TextureDescriptor {
                                    label: Some("PixelBenderShader target clone"),
                                    size: extent,
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: wgpu::TextureFormat::Rgba8Unorm,
                                    usage: wgpu::TextureUsages::COPY_DST
                                        | wgpu::TextureUsages::TEXTURE_BINDING,
                                    view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                                });
                            render_command_encoder.copy_texture_to_texture(
                                ImageCopyTexture {
                                    texture: &target.texture,
                                    mip_level: 0,
                                    origin: Default::default(),
                                    aspect: Default::default(),
                                },
                                ImageCopyTexture {
                                    texture: &fresh_texture,
                                    mip_level: 0,
                                    origin: Default::default(),
                                    aspect: Default::default(),
                                },
                                extent,
                            );

                            BitmapHandle(Arc::new(Texture {
                                texture: Arc::new(fresh_texture),
                                bind_linear: Default::default(),
                                bind_nearest: Default::default(),
                                copy_count: Cell::new(0),
                            }))
                        });
                        *texture = cached_fresh_handle.clone();
                    }
                    texture_views.insert(
                        *index,
                        as_texture(texture)
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    );
                }
                PixelBenderShaderArgument::ValueInput { index, value } => {
                    let param = &compiled_shader.shader.params[*index as usize];

                    let name = match param {
                        PixelBenderParam::Normal { name, .. } => name,
                        _ => unreachable!(),
                    };

                    if name == OUT_COORD_NAME {
                        continue;
                    }

                    let (value_vec, is_float): ([f32; 4], bool) = match value {
                        PixelBenderType::TFloat(f1) => ([*f1, 0.0, 0.0, 0.0], true),
                        PixelBenderType::TFloat2(f1, f2) => ([*f1, *f2, 0.0, 0.0], true),
                        PixelBenderType::TFloat3(f1, f2, f3) => ([*f1, *f2, *f3, 0.0], true),
                        PixelBenderType::TFloat4(f1, f2, f3, f4) => ([*f1, *f2, *f3, *f4], true),
                        PixelBenderType::TInt(i1) => ([*i1 as f32, 0.0, 0.0, 0.0], false),
                        PixelBenderType::TInt2(i1, i2) => {
                            ([*i1 as f32, *i2 as f32, 0.0, 0.0], false)
                        }
                        PixelBenderType::TInt3(i1, i2, i3) => {
                            ([*i1 as f32, *i2 as f32, *i3 as f32, 0.0], false)
                        }
                        PixelBenderType::TInt4(i1, i2, i3, i4) => {
                            ([*i1 as f32, *i2 as f32, *i3 as f32, *i4 as f32], false)
                        }
                        _ => unreachable!("Unimplemented value {value:?}"),
                    };

                    // Both float32 and int are 4 bytes
                    let component_size_bytes = 4;

                    let (buffer, vec4_count) = if is_float {
                        let res = (&compiled_shader.float_parameters_buffer, float_offset);
                        float_offset += 1;
                        res
                    } else {
                        let res = (&compiled_shader.int_parameters_buffer, int_offset);
                        int_offset += 1;
                        res
                    };

                    let mut buffer_slice = staging_belt.write_buffer(
                        &mut render_command_encoder,
                        buffer,
                        vec4_count * 4 * component_size_bytes,
                        NonZeroU64::new(4 * component_size_bytes).unwrap(),
                        &self.descriptors.device,
                    );
                    buffer_slice.copy_from_slice(bytemuck::cast_slice(&value_vec));
                }
            }
        }

        // This needs to be a separate loop, so that we can get references into `texture_views`
        for input in &arguments {
            match input {
                PixelBenderShaderArgument::ImageInput { index, .. } => {
                    let binding = naga_pixelbender::TEXTURE_START_BIND_INDEX + *index as u32;
                    bind_group_entries.push(BindGroupEntry {
                        binding,
                        resource: BindingResource::TextureView(&texture_views[index]),
                    });
                }
                PixelBenderShaderArgument::ValueInput { .. } => {}
            }
        }

        let bind_group = self
            .descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &compiled_shader.bind_group_layout,
                entries: &bind_group_entries,
            });

        staging_belt.finish();

        let mut render_pass =
            render_command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PixelBender render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame_output.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_pipeline(&compiled_shader.pipeline);

        render_pass.set_vertex_buffer(0, self.descriptors.quad.vertices_pos.slice(..));
        render_pass.set_index_buffer(
            self.descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..6, 0, 0..1);

        drop(render_pass);

        self.descriptors
            .queue
            .submit(Some(render_command_encoder.finish()));

        staging_belt.recall();

        Ok(Box::new(QueueSyncHandle::NotCopied {
            handle: target_handle,
            copy_area: PixelRegion::for_whole_size(extent.width, extent.height),
            descriptors: self.descriptors.clone(),
            pool: self.offscreen_buffer_pool.clone(),
        }))
    }
}
