use crate::{Error, MaskState, Vertex};
use enum_map::{enum_map, EnumMap};
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct ShapePipeline {
    pub mask_pipelines: EnumMap<MaskState, wgpu::RenderPipeline>,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color_pipelines: ShapePipeline,

    pub bitmap_pipelines: ShapePipeline,
    pub bitmap_layout: wgpu::BindGroupLayout,

    pub gradient_pipelines: ShapePipeline,
    pub gradient_layout: wgpu::BindGroupLayout,
}

impl ShapePipeline {
    pub fn pipeline_for(&self, mask_state: MaskState) -> &wgpu::RenderPipeline {
        &self.mask_pipelines[mask_state]
    }
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        msaa_sample_count: u32,
        sampler_layout: &wgpu::BindGroupLayout,
        globals_layout: &wgpu::BindGroupLayout,
        dynamic_uniforms_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, Error> {
        // TODO: Naga validation errors when encountering push constants currently.
        // Disable validation for now. Remove this when Naga can swallow it.
        macro_rules! include_spirv {
            ($($token:tt)*) => {
                {
                    wgpu::ShaderModuleDescriptor {
                        label: Some($($token)*),
                        source: wgpu::util::make_spirv(include_bytes!($($token)*)),
                    }
                }
            };
        }

        let color_vs = device.create_shader_module(&include_spirv!("../shaders/color.vert.spv"));
        let color_fs = device.create_shader_module(&include_spirv!("../shaders/color.frag.spv"));
        let texture_vs =
            device.create_shader_module(&include_spirv!("../shaders/texture.vert.spv"));
        let gradient_fs =
            device.create_shader_module(&include_spirv!("../shaders/gradient.frag.spv"));
        let bitmap_fs = device.create_shader_module(&include_spirv!("../shaders/bitmap.frag.spv"));

        let vertex_buffers_description = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Float32x4,
            ],
        }];

        let color_pipelines = create_color_pipelines(
            device,
            &color_vs,
            &color_fs,
            msaa_sample_count,
            &vertex_buffers_description,
            globals_layout,
            dynamic_uniforms_layout,
        );

        let bitmap_bind_layout_label = create_debug_label!("Bitmap shape bind group layout");
        let bitmap_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: bitmap_bind_layout_label.as_deref(),
            });

        let bitmap_pipelines = create_bitmap_pipeline(
            device,
            &texture_vs,
            &bitmap_fs,
            msaa_sample_count,
            &vertex_buffers_description,
            sampler_layout,
            globals_layout,
            dynamic_uniforms_layout,
            &bitmap_bind_layout,
        );

        let gradient_bind_layout_label = create_debug_label!("Gradient shape bind group");
        let gradient_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: gradient_bind_layout_label.as_deref(),
            });

        let gradient_pipelines = create_gradient_pipeline(
            device,
            &texture_vs,
            &gradient_fs,
            msaa_sample_count,
            &vertex_buffers_description,
            globals_layout,
            dynamic_uniforms_layout,
            &gradient_bind_layout,
        );

        Ok(Self {
            color_pipelines,
            bitmap_pipelines,
            bitmap_layout: bitmap_bind_layout,
            gradient_pipelines,
            gradient_layout: gradient_bind_layout,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn create_pipeline_descriptor<'a>(
    label: Option<&'a str>,
    vertex_shader: &'a wgpu::ShaderModule,
    fragment_shader: &'a wgpu::ShaderModule,
    pipeline_layout: &'a wgpu::PipelineLayout,
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    color_target_state: &'a [wgpu::ColorTargetState],
    vertex_buffer_layout: &'a [wgpu::VertexBufferLayout<'a>],
    msaa_sample_count: u32,
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: "main",
            buffers: vertex_buffer_layout,
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: "main",
            targets: color_target_state,
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::default(),
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: depth_stencil_state,
        multisample: wgpu::MultisampleState {
            count: msaa_sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    }
}

fn create_color_pipelines(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_description: &[wgpu::VertexBufferLayout<'_>],
    globals_layout: &wgpu::BindGroupLayout,
    dynamic_uniforms_layout: &wgpu::BindGroupLayout,
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("Color shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[globals_layout, dynamic_uniforms_layout],
        push_constant_ranges: &[],
    });

    let mask_pipelines = enum_map! {
        MaskState::NoMask => {
            let (stencil, write_mask) = mask_render_state(MaskState::NoMask);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Color pipeline no mask").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,

                        }
                    }),
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        },

        MaskState::DrawMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Color pipeline draw mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        },

        MaskState::DrawMaskedContent => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskedContent);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Color pipeline draw masked content").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        },

        MaskState::ClearMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::ClearMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Color pipeline clear mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        },
    };

    ShapePipeline { mask_pipelines }
}

#[allow(clippy::too_many_arguments)]
fn create_bitmap_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_layout: &[wgpu::VertexBufferLayout<'_>],
    sampler_layout: &wgpu::BindGroupLayout,
    globals_layout: &wgpu::BindGroupLayout,
    dynamic_uniforms_layout: &wgpu::BindGroupLayout,
    bitmap_bind_layout: &wgpu::BindGroupLayout,
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("Bitmap shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[
            globals_layout,
            dynamic_uniforms_layout,
            bitmap_bind_layout,
            sampler_layout,
        ],
        push_constant_ranges: &[],
    });

    let mask_pipelines = enum_map! {
        MaskState::NoMask => {
            let (stencil, write_mask) = mask_render_state(MaskState::NoMask);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Bitmap pipeline no mask").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },

        MaskState::DrawMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Bitmap pipeline draw mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },

        MaskState::DrawMaskedContent => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskedContent);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Bitmap pipeline draw masked content").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Equal,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },

        MaskState::ClearMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::ClearMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Bitmap pipeline clear mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        }
    };

    ShapePipeline { mask_pipelines }
}

#[allow(clippy::too_many_arguments)]
fn create_gradient_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_layout: &[wgpu::VertexBufferLayout<'_>],
    globals_layout: &wgpu::BindGroupLayout,
    dynamic_uniforms_layout: &wgpu::BindGroupLayout,
    gradient_bind_layout: &wgpu::BindGroupLayout,
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("Gradient shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[
            globals_layout,
            dynamic_uniforms_layout,
            gradient_bind_layout,
        ],
        push_constant_ranges: &[],
    });

    let mask_pipelines = enum_map! {
        MaskState::NoMask => {
            let (stencil, write_mask) = mask_render_state(MaskState::NoMask);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Gradient pipeline no mask").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },

        MaskState::DrawMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Gradient pipeline draw mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },


        MaskState::DrawMaskedContent => {
            let (stencil, write_mask) = mask_render_state(MaskState::DrawMaskedContent);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Gradient pipeline draw masked content").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Equal,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        },

        MaskState::ClearMaskStencil => {
            let (stencil, write_mask) = mask_render_state(MaskState::ClearMaskStencil);
            device.create_render_pipeline(&create_pipeline_descriptor(
                create_debug_label!("Gradient pipeline clear mask stencil").as_deref(),
                vertex_shader,
                fragment_shader,
                &pipeline_layout,
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                    bias: Default::default(),
                }),
                &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask,
                }],
                vertex_buffers_layout,
                msaa_sample_count,
            ))
        }
    };

    ShapePipeline { mask_pipelines }
}

fn mask_render_state(state: MaskState) -> (wgpu::StencilState, wgpu::ColorWrites) {
    let (stencil_state, color_write) = match state {
        MaskState::NoMask => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrites::ALL,
        ),
        MaskState::DrawMaskStencil => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::IncrementClamp,
            },
            wgpu::ColorWrites::empty(),
        ),
        MaskState::DrawMaskedContent => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrites::ALL,
        ),
        MaskState::ClearMaskStencil => (
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::DecrementClamp,
            },
            wgpu::ColorWrites::empty(),
        ),
    };

    (
        wgpu::StencilState {
            front: stencil_state,
            back: stencil_state,
            read_mask: 0xff,
            write_mask: 0xff,
        },
        color_write,
    )
}
