use crate::layouts::BindLayouts;
use crate::pipelines::VERTEX_BUFFERS_DESCRIPTION;
use crate::shaders::Shaders;
use crate::{
    create_buffer_with_data, BitmapSamplers, Pipelines, TextureTransforms, Transforms, Vertex,
    DEFAULT_COLOR_ADJUSTMENTS,
};
use fnv::FnvHashMap;
use std::mem;
use std::sync::{Arc, Mutex};

pub struct Descriptors {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub limits: wgpu::Limits,
    pub queue: wgpu::Queue,
    pub bitmap_samplers: BitmapSamplers,
    pub bind_layouts: BindLayouts,
    pub quad: Quad,
    copy_pipeline: Mutex<FnvHashMap<wgpu::TextureFormat, Arc<wgpu::RenderPipeline>>>,
    copy_srgb_pipeline: Mutex<FnvHashMap<wgpu::TextureFormat, Arc<wgpu::RenderPipeline>>>,
    shaders: Shaders,
    pipelines: Mutex<FnvHashMap<(u32, wgpu::TextureFormat), Arc<Pipelines>>>,
    pub default_color_bind_group: wgpu::BindGroup,
    pub empty_bind_group: wgpu::BindGroup,
}

impl Descriptors {
    pub fn new(adapter: wgpu::Adapter, device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let limits = device.limits();
        let bind_layouts = BindLayouts::new(&device);
        let bitmap_samplers = BitmapSamplers::new(&device);
        let shaders = Shaders::new(&device);
        let quad = Quad::new(&device);
        let default_color_transform = create_buffer_with_data(
            &device,
            bytemuck::cast_slice(&[DEFAULT_COLOR_ADJUSTMENTS]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Default colors"),
        );
        let default_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: create_debug_label!("Default colors").as_deref(),
            layout: &bind_layouts.color_transforms,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: default_color_transform.as_entire_binding(),
            }],
        });

        let empty_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_layouts.transforms,
            entries: &[],
        });

        Self {
            adapter,
            device,
            limits,
            queue,
            bitmap_samplers,
            bind_layouts,
            quad,
            copy_pipeline: Default::default(),
            copy_srgb_pipeline: Default::default(),
            shaders,
            pipelines: Default::default(),
            default_color_bind_group,
            empty_bind_group,
        }
    }

    pub fn copy_srgb_pipeline(&self, format: wgpu::TextureFormat) -> Arc<wgpu::RenderPipeline> {
        let mut pipelines = self
            .copy_srgb_pipeline
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry(format)
            .or_insert_with(|| {
                let copy_texture_pipeline_layout =
                    &self
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: create_debug_label!("Copy sRGB pipeline layout").as_deref(),
                            bind_group_layouts: &[
                                &self.bind_layouts.globals,
                                &self.bind_layouts.transforms,
                                &self.bind_layouts.bitmap,
                            ],
                            push_constant_ranges: if self.device.limits().max_push_constant_size > 0
                            {
                                &[wgpu::PushConstantRange {
                                    stages: wgpu::ShaderStages::VERTEX,
                                    range: 0..(mem::size_of::<Transforms>() as u32),
                                }]
                            } else {
                                &[]
                            },
                        });
                Arc::new(
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: create_debug_label!("Copy sRGB pipeline").as_deref(),
                            layout: Some(&copy_texture_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &self.shaders.copy_srgb_shader,
                                entry_point: "main_vertex",
                                buffers: &VERTEX_BUFFERS_DESCRIPTION,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &self.shaders.copy_srgb_shader,
                                entry_point: "main_fragment",
                                targets: &[Some(wgpu::ColorTargetState {
                                    format,
                                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                                    write_mask: Default::default(),
                                })],
                            }),
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
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                        }),
                )
            })
            .clone()
    }

    pub fn copy_pipeline(&self, format: wgpu::TextureFormat) -> Arc<wgpu::RenderPipeline> {
        let mut pipelines = self
            .copy_pipeline
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry(format)
            .or_insert_with(|| {
                let copy_texture_pipeline_layout =
                    &self
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: create_debug_label!("Copy pipeline layout").as_deref(),
                            bind_group_layouts: &[
                                &self.bind_layouts.globals,
                                &self.bind_layouts.transforms,
                                &self.bind_layouts.bitmap,
                            ],
                            push_constant_ranges: if self.device.limits().max_push_constant_size > 0
                            {
                                &[wgpu::PushConstantRange {
                                    stages: wgpu::ShaderStages::VERTEX,
                                    range: 0..(mem::size_of::<Transforms>() as u32),
                                }]
                            } else {
                                &[]
                            },
                        });
                Arc::new(
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: create_debug_label!("Copy pipeline").as_deref(),
                            layout: Some(&copy_texture_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &self.shaders.copy_shader,
                                entry_point: "main_vertex",
                                buffers: &VERTEX_BUFFERS_DESCRIPTION,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &self.shaders.copy_shader,
                                entry_point: "main_fragment",
                                targets: &[Some(wgpu::ColorTargetState {
                                    format,
                                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                                    write_mask: Default::default(),
                                })],
                            }),
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
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                        }),
                )
            })
            .clone()
    }

    pub fn pipelines(&self, msaa_sample_count: u32, format: wgpu::TextureFormat) -> Arc<Pipelines> {
        let mut pipelines = self
            .pipelines
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry((msaa_sample_count, format))
            .or_insert_with(|| {
                Arc::new(Pipelines::new(
                    &self.device,
                    &self.shaders,
                    format,
                    msaa_sample_count,
                    &self.bind_layouts,
                ))
            })
            .clone()
    }
}

pub struct Quad {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub texture_transforms: wgpu::Buffer,
}

impl Quad {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = [
            Vertex {
                position: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vbo = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Quad vbo"),
        );

        let ibo = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&indices),
            wgpu::BufferUsages::INDEX,
            create_debug_label!("Quad ibo"),
        );

        let tex_transforms = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&[TextureTransforms {
                u_matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Quad tex transforms"),
        );

        Self {
            vertices: vbo,
            indices: ibo,
            texture_transforms: tex_transforms,
        }
    }
}
