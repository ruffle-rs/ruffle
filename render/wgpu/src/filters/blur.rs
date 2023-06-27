use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::{FilterSource, VERTEX_BUFFERS_DESCRIPTION_FILTERS};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use std::sync::OnceLock;
use swf::BlurFilter as BlurFilterArgs;
use wgpu::util::DeviceExt;

pub struct BlurFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipelines: SampleCountMap<OnceLock<wgpu::RenderPipeline>>,
}

impl BlurFilter {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 4]>() as u64
                        ),
                    },
                    count: None,
                },
            ],
            label: create_debug_label!("Blur filter binds").as_deref(),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
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

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        source: FilterSource,
        filter: &BlurFilterArgs,
    ) -> CommandTarget {
        let sample_count = source.texture.sample_count();
        let format = source.texture.format();
        let pipeline = self.pipeline(descriptors, sample_count);

        // FIXME - this should be larger than the source texture. Figure out exactly how much larger
        let targets = [
            CommandTarget::new(
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
            ),
            CommandTarget::new(
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
            ),
        ];

        // TODO: Vertices should be per pass, and each pass needs diff sizes
        let vertices = source.vertices(&descriptors.device);

        let source_view = source.texture.create_view(&Default::default());
        for i in 0..2 {
            let blur_x = (filter.blur_x.to_f32() - 1.0).max(0.0);
            let blur_y = (filter.blur_y.to_f32() - 1.0).max(0.0);
            let current = &targets[i % 2];
            let (previous_view, previous_vertices, previous_width, previous_height) = if i == 0 {
                (
                    &source_view,
                    vertices.slice(..),
                    source.texture.width() as f32,
                    source.texture.height() as f32,
                )
            } else {
                let previous = &targets[(i - 1) % 2];
                (
                    previous.color_view(),
                    descriptors.quad.filter_vertices.slice(..),
                    previous.width() as f32,
                    previous.height() as f32,
                )
            };
            let buffer = descriptors
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: create_debug_label!("Filter arguments").as_deref(),
                    contents: bytemuck::cast_slice(&[
                        blur_x * ((i as u32) % 2) as f32,
                        blur_y * (((i as u32) % 2) + 1) as f32,
                        previous_width,
                        previous_height,
                    ]),
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
                            resource: wgpu::BindingResource::TextureView(previous_view),
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
                color_attachments: &[current.color_attachments()],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(pipeline);

            render_pass.set_bind_group(0, &filter_group, &[]);

            render_pass.set_vertex_buffer(0, previous_vertices);
            render_pass.set_index_buffer(
                descriptors.quad.indices.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        targets
            .into_iter()
            .last()
            .expect("Targets should not be empty")
    }
}
