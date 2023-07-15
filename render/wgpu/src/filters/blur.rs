use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::{FilterSource, VERTEX_BUFFERS_DESCRIPTION_FILTERS};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;
use swf::{BlurFilter as BlurFilterArgs, Rectangle};
use wgpu::util::DeviceExt;
use wgpu::{BufferSlice, CommandEncoder, RenderPipeline, TextureView};

/// How much each pass should multiply the requested blur size by - accumulative.
/// These are very approximate to Flash, and not 100% exact.
/// Pass 1 would be 100%, but pass 2 would be 110%.
/// This is accumulative so you can calculate the size upfront for how many passes you'll need to perform.
const PASS_SCALES: [f32; 15] = [
    1.0, 2.1, 2.7, 3.1, 3.5, 3.8, 4.0, 4.2, 4.4, 4.6, 5.0, 6.0, 6.0, 7.0, 7.0,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct BlurUniform {
    direction: [f32; 2],
    full_size: f32,
    left_weight: f32,
}

pub struct BlurFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipelines: SampleCountMap<OnceLock<wgpu::RenderPipeline>>,
}

impl BlurFilter {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };
        let sampling = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let bind_group_layout = if device.limits().max_push_constant_size > 0 {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[texture, sampling],
                label: create_debug_label!("Blur filter binds").as_deref(),
            })
        } else {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    texture,
                    sampling,
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<BlurUniform>() as u64,
                            ),
                        },
                        count: None,
                    },
                ],
                label: create_debug_label!("Blur filter binds (with buffer)").as_deref(),
            })
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: if device.limits().max_push_constant_size > 0 {
                &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: 0..(std::mem::size_of::<BlurUniform>() as u32),
                }]
            } else {
                &[]
            },
        });

        Self {
            pipelines: Default::default(),
            pipeline_layout,
            bind_group_layout,
        }
    }

    fn pipeline(&self, descriptors: &Descriptors, msaa_sample_count: u32) -> &wgpu::RenderPipeline {
        self.pipelines.get_or_init(msaa_sample_count, || {
            let label = create_debug_label!("Blur Filter ({} msaa)", msaa_sample_count);
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: label.as_deref(),
                    layout: Some(&self.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.blur_filter,
                        entry_point: "main_vertex",
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_FILTERS,
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::default(),
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: msaa_sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &descriptors.shaders.blur_filter,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                    }),
                    multiview: None,
                })
        })
    }

    pub fn calculate_dest_rect(
        &self,
        filter: &BlurFilterArgs,
        source_rect: Rectangle<i32>,
    ) -> Rectangle<i32> {
        let scale = PASS_SCALES[filter.num_passes().clamp(1, 15) as usize - 1];
        let x = (scale * filter.blur_x.to_f32()).ceil().max(0.0) as i32;
        let y = (scale * filter.blur_y.to_f32()).ceil().max(0.0) as i32;
        Rectangle {
            x_min: source_rect.x_min - x,
            x_max: source_rect.x_max + x,
            y_min: source_rect.y_min - y,
            y_max: source_rect.y_max + y,
        }
    }

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        source: &FilterSource,
        filter: &BlurFilterArgs,
    ) -> Option<CommandTarget> {
        let sample_count = source.texture.sample_count();
        let format = source.texture.format();
        let pipeline = self.pipeline(descriptors, sample_count);

        let mut flip = CommandTarget::new(
            descriptors,
            texture_pool,
            wgpu::Extent3d {
                width: source.size.0,
                height: source.size.1,
                depth_or_array_layers: 1,
            },
            format,
            sample_count,
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            draw_encoder,
        );
        let mut flop = CommandTarget::new(
            descriptors,
            texture_pool,
            wgpu::Extent3d {
                width: source.size.0,
                height: source.size.1,
                depth_or_array_layers: 1,
            },
            format,
            sample_count,
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            draw_encoder,
        );

        let vertices = source.vertices(&descriptors.device);

        let source_view = source.texture.create_view(&Default::default());
        let mut first = true;
        let mut last_scale_total = 0.0;
        for current_scale_total in PASS_SCALES.into_iter().take(filter.num_passes() as usize) {
            let pass_scale = current_scale_total - last_scale_total;
            last_scale_total = current_scale_total;

            for i in 0..2 {
                let horizontal = i % 2 == 0;
                let strength = if horizontal {
                    filter.blur_x.to_f32()
                } else {
                    filter.blur_y.to_f32()
                };
                // Full width of the kernel (left edge to right edge)
                let full_size = (strength.min(255.0) * pass_scale).round();
                if full_size <= 1.0 {
                    // A width of 1 or less is a noop (it'd just sample itself and nothing else)
                    continue;
                }

                let (previous_view, previous_vertices, previous_width, previous_height) = if first {
                    first = false;
                    (
                        &source_view,
                        vertices.slice(..),
                        source.texture.width() as f32,
                        source.texture.height() as f32,
                    )
                } else {
                    (
                        flip.color_view(),
                        descriptors.quad.filter_vertices.slice(..),
                        flip.width() as f32,
                        flip.height() as f32,
                    )
                };
                let uniform = BlurUniform {
                    direction: if horizontal {
                        [1.0 / previous_width, 0.0]
                    } else {
                        [0.0, 1.0 / previous_height]
                    },
                    full_size,
                    left_weight: ((full_size % 2.0) * 0.5) + 0.5,
                };

                if descriptors.limits.max_push_constant_size > 0 {
                    self.render_with_push_constants(
                        descriptors,
                        draw_encoder,
                        pipeline,
                        &mut flop,
                        previous_view,
                        previous_vertices,
                        uniform,
                    );
                } else {
                    self.render_with_uniform_buffers(
                        descriptors,
                        draw_encoder,
                        pipeline,
                        &mut flop,
                        previous_view,
                        previous_vertices,
                        uniform,
                    );
                }

                std::mem::swap(&mut flip, &mut flop);
            }
        }

        if first {
            // Nothing happened, don't return an empty unused texture
            None
        } else {
            Some(flip)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_with_push_constants(
        &self,
        descriptors: &Descriptors,
        draw_encoder: &mut CommandEncoder,
        pipeline: &RenderPipeline,
        destination: &mut CommandTarget,
        source: &TextureView,
        vertices: BufferSlice,
        uniform: BlurUniform,
    ) {
        let filter_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Filter group").as_deref(),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            descriptors.bitmap_samplers.get_sampler(false, true),
                        ),
                    },
                ],
            });

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Blur filter").as_deref(),
            color_attachments: &[destination.color_attachments()],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(pipeline);

        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::cast_slice(&[uniform]),
        );

        render_pass.set_bind_group(0, &filter_group, &[]);

        render_pass.set_vertex_buffer(0, vertices);
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_with_uniform_buffers(
        &self,
        descriptors: &Descriptors,
        draw_encoder: &mut CommandEncoder,
        pipeline: &RenderPipeline,
        destination: &mut CommandTarget,
        source: &TextureView,
        vertices: BufferSlice,
        uniform: BlurUniform,
    ) {
        let buffer = descriptors
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: create_debug_label!("Filter arguments").as_deref(),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let filter_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Filter group").as_deref(),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            descriptors.bitmap_samplers.get_sampler(false, true),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            });

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Blur filter").as_deref(),
            color_attachments: &[destination.color_attachments()],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, &filter_group, &[]);

        render_pass.set_vertex_buffer(0, vertices);
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
