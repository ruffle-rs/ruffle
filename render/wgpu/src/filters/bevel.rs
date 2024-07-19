use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::blur::BlurFilter;
use crate::filters::{
    FilterSource, FilterVertexWithDoubleBlur, VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_DOUBLE_BLUR,
};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;
use swf::BevelFilter as BevelFilterArgs;
use wgpu::util::StagingBelt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct BevelUniform {
    highlight_color: [f32; 4],
    shadow_color: [f32; 4],
    strength: f32,
    bevel_type: u32,       // 0 outer, 1 inner, 2 full
    knockout: u32,         // a wasteful bool, but we need to be aligned anyway
    composite_source: u32, // undocumented flash feature, another bool
}

pub struct BevelFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    vertices_size: wgpu::BufferSize,
    uniform_size: wgpu::BufferSize,
    pipeline: SampleCountMap<OnceLock<wgpu::RenderPipeline>>,
}

impl BevelFilter {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<BevelUniform>() as u64;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: create_debug_label!("Bevel filter binds").as_deref(),
        });

        let vertices_size = std::mem::size_of::<[FilterVertexWithDoubleBlur; 4]>() as u64;
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
            pipeline: Default::default(),
            pipeline_layout,
            vertex_buffer,
            uniform_buffer,
            bind_group_layout,
            uniform_size: wgpu::BufferSize::new(uniform_size).expect("Definitely not zero."),
            vertices_size: wgpu::BufferSize::new(vertices_size).expect("Definitely not zero."),
        }
    }

    fn pipeline(&self, descriptors: &Descriptors, msaa_sample_count: u32) -> &wgpu::RenderPipeline {
        self.pipeline.get_or_init(msaa_sample_count, || {
            let label = create_debug_label!("Bevel Filter ({} msaa)", msaa_sample_count);
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: label.as_deref(),
                    layout: Some(&self.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.bevel_filter,
                        entry_point: "main_vertex",
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_DOUBLE_BLUR,
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
                        module: &descriptors.shaders.bevel_filter,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                })
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut StagingBelt,
        source: &FilterSource,
        filter: &BevelFilterArgs,
        blur_filter: &BlurFilter,
    ) -> CommandTarget {
        let sample_count = source.texture.sample_count();
        let format = source.texture.format();
        let pipeline = self.pipeline(descriptors, sample_count);
        let blurred = blur_filter.apply(
            descriptors,
            texture_pool,
            draw_encoder,
            staging_belt,
            source,
            &filter.inner_blur_filter(),
        );
        let blurred_texture = if let Some(blurred) = &blurred {
            blurred.ensure_cleared(draw_encoder);
            blurred.color_texture()
        } else {
            source.texture
        };
        let source_view = source.texture.create_view(&Default::default());
        let blurred_view = blurred_texture.create_view(&Default::default());
        let distance = filter.distance.to_f32();
        let angle = filter.angle.to_f32();
        let blur_offset = (angle.cos() * distance, angle.sin() * distance);

        let target = CommandTarget::new(
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
        let mut highlight_color = [
            f32::from(filter.highlight_color.r) / 255.0,
            f32::from(filter.highlight_color.g) / 255.0,
            f32::from(filter.highlight_color.b) / 255.0,
            f32::from(filter.highlight_color.a) / 255.0,
        ];
        highlight_color[0] *= highlight_color[3];
        highlight_color[1] *= highlight_color[3];
        highlight_color[2] *= highlight_color[3];
        let mut shadow_color = [
            f32::from(filter.shadow_color.r) / 255.0,
            f32::from(filter.shadow_color.g) / 255.0,
            f32::from(filter.shadow_color.b) / 255.0,
            f32::from(filter.shadow_color.a) / 255.0,
        ];
        shadow_color[0] *= shadow_color[3];
        shadow_color[1] *= shadow_color[3];
        shadow_color[2] *= shadow_color[3];
        staging_belt
            .write_buffer(
                draw_encoder,
                &self.uniform_buffer,
                0,
                self.uniform_size,
                &descriptors.device,
            )
            .copy_from_slice(bytemuck::cast_slice(&[BevelUniform {
                highlight_color,
                shadow_color,
                strength: filter.strength.to_f32(),
                bevel_type: if filter.is_on_top() {
                    2
                } else if filter.is_inner() {
                    1
                } else {
                    0
                },
                knockout: if filter.is_knockout() { 1 } else { 0 },
                composite_source: 1,
            }]));
        staging_belt
            .write_buffer(
                draw_encoder,
                &self.vertex_buffer,
                0,
                self.vertices_size,
                &descriptors.device,
            )
            .copy_from_slice(bytemuck::cast_slice(&[
                source.vertices_with_highlight_and_shadow(blur_offset)
            ]));
        let filter_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Filter group").as_deref(),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            descriptors.bitmap_samplers.get_sampler(false, false),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&blurred_view),
                    },
                ],
            });
        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Bevel filter").as_deref(),
            color_attachments: &[target.color_attachments()],
            ..Default::default()
        });
        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, &filter_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);
        target
    }
}
