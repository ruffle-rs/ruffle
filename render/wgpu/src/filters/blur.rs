use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::{FilterSource, FilterVertex, VERTEX_BUFFERS_DESCRIPTION_FILTERS};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;
use swf::BlurFilter as BlurFilterArgs;
use wgpu::util::StagingBelt;
use wgpu::{BufferSlice, CommandEncoder, RenderPipeline, TextureView};

/// This is a 1:1 match of `struct Filter` in `blur.wgsl`. See that, and the usage below, for more info.
/// Since WebGL requires 16 byte struct size (alignment), some of these fields (namely m2 and last_weight)
/// are passed in precomputed, even though they are trivial to get (addition/multiplication by constant).
/// The struct would have to be padded with dummy data otherwise anyway - these are at least useful.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct BlurUniform {
    direction: [f32; 2],
    full_size: f32,
    m: f32,
    m2: f32,
    first_weight: f32,
    last_offset: f32,
    last_weight: f32,
}

pub struct BlurFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    vertices_size: wgpu::BufferSize,
    uniform_size: wgpu::BufferSize,
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
        let uniform_size = std::mem::size_of::<BlurUniform>() as u64;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                texture,
                sampling,
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_size),
                    },
                    count: None,
                },
            ],
            label: create_debug_label!("Blur filter binds (with buffer)").as_deref(),
        });

        let vertices_size = std::mem::size_of::<[FilterVertex; 4]>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertices_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Self {
            pipelines: Default::default(),
            pipeline_layout,
            vertex_buffer,
            uniform_buffer,
            bind_group_layout,
            vertices_size: wgpu::BufferSize::new(vertices_size).expect("Definitely not zero."),
            uniform_size: wgpu::BufferSize::new(uniform_size).expect("Definitely not zero."),
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
                        compilation_options: Default::default(),
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
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                })
        })
    }

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut StagingBelt,
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

        staging_belt
            .write_buffer(
                draw_encoder,
                &self.vertex_buffer,
                0,
                self.vertices_size,
                &descriptors.device,
            )
            .copy_from_slice(bytemuck::cast_slice(&[source.vertices()]));

        let source_view = source.texture.create_view(&Default::default());
        let mut first = true;
        for _ in 0..(filter.num_passes() as usize) {
            for i in 0..2 {
                let horizontal = i % 2 == 0;
                let strength = if horizontal {
                    filter.blur_x.to_f32()
                } else {
                    filter.blur_y.to_f32()
                };
                // Full width of the kernel (left edge to right edge)
                let full_size = strength.min(255.0);
                if full_size <= 1.0 {
                    // A width of 1 or less is a noop (it'd just sample itself and nothing else)
                    continue;
                }

                let (previous_view, previous_vertices, previous_width, previous_height) = if first {
                    first = false;
                    (
                        &source_view,
                        self.vertex_buffer.slice(..),
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

                // See this article for additional information on the fractional blur algorithm, as this
                // implementation was inspired by it: https://fgiesen.wordpress.com/2012/08/01/fast-blurs-2/

                // This is how much the blur "extends past" the center pixel to either side.
                let radius = (full_size - 1.0) / 2.0;

                // This is how many simple double-1 weighted pixel pairs we can sample in the center.
                // Note how we're not using floor() here. This is to guarantee that alpha is not 0 when
                // radius is a whole number: That would cause the division below to end the universe,
                // and more importantly, also waste at least one sampling of the texture (the first one).
                // This way, alpha is 1 instead in those cases (with m being one smaller), and the last
                // two samplings can be fused into one, at the right place and with the right weight.
                let m = radius.ceil() - 1.0;
                // Not the transparency kind. It's almost the fractional part of radius.
                // If radius is a whole number, however, it's 1 instead of 0.
                // The rounding is done to imitate the fixed-point calculations in Flash Player,
                // improving emulation accuracy somewhat.
                let alpha = ((radius - m) * 255.0).floor() / 255.0;

                // These control how and where the last pair of pixels are to be sampled,
                // so that the next-to-last will end up with an effective weight of 1.0,
                // and the last one with a weight of alpha. Note that the offset is relative
                // to the center of the next-to-last sampled pixel, in the range of 0 to 0.5.
                let last_offset = 1.0 / ((1.0 / alpha) + 1.0);
                let last_weight = alpha + 1.0;

                let uniform = BlurUniform {
                    direction: if horizontal {
                        [1.0 / previous_width, 0.0]
                    } else {
                        [0.0, 1.0 / previous_height]
                    },
                    full_size,
                    m,
                    m2: m * 2.0,
                    first_weight: alpha,
                    last_offset,
                    last_weight,
                };
                staging_belt
                    .write_buffer(
                        draw_encoder,
                        &self.uniform_buffer,
                        0,
                        self.uniform_size,
                        &descriptors.device,
                    )
                    .copy_from_slice(bytemuck::cast_slice(&[uniform]));

                self.render_with_uniform_buffers(
                    descriptors,
                    draw_encoder,
                    pipeline,
                    &mut flop,
                    previous_view,
                    previous_vertices,
                );

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
    fn render_with_uniform_buffers(
        &self,
        descriptors: &Descriptors,
        draw_encoder: &mut CommandEncoder,
        pipeline: &RenderPipeline,
        destination: &mut CommandTarget,
        source: &TextureView,
        vertices: BufferSlice,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Blur filter").as_deref(),
            color_attachments: &[destination.color_attachments()],
            ..Default::default()
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
