use crate::{Error, GPUVertex, MaskState};
use enum_map::{enum_map, EnumMap};
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct ShapePipeline {
    pub mask_pipelines: EnumMap<MaskState, wgpu::RenderPipeline>,
    pub bind_layout: wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color: ShapePipeline,
    pub bitmap: ShapePipeline,
    pub gradient: ShapePipeline,
}

impl ShapePipeline {
    pub fn pipeline_for(&self, mask_state: MaskState) -> &wgpu::RenderPipeline {
        &self.mask_pipelines[mask_state]
    }
}

impl Pipelines {
    pub fn new(device: &wgpu::Device, msaa_sample_count: u32) -> Result<Self, Error> {
        let color_vs =
            device.create_shader_module(wgpu::include_spirv!("../shaders/color.vert.spv"));
        let color_fs =
            device.create_shader_module(wgpu::include_spirv!("../shaders/color.frag.spv"));
        let texture_vs =
            device.create_shader_module(wgpu::include_spirv!("../shaders/texture.vert.spv"));
        let gradient_fs =
            device.create_shader_module(wgpu::include_spirv!("../shaders/gradient.frag.spv"));
        let bitmap_fs =
            device.create_shader_module(wgpu::include_spirv!("../shaders/bitmap.frag.spv"));

        let vertex_buffers_description = [wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<GPUVertex>() as u64,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float2,
                1 => Float4
            ],
        }];

        Ok(Self {
            color: create_color_pipelines(
                &device,
                &color_vs,
                &color_fs,
                msaa_sample_count,
                &vertex_buffers_description,
            ),
            bitmap: create_bitmap_pipeline(
                &device,
                &texture_vs,
                &bitmap_fs,
                msaa_sample_count,
                &vertex_buffers_description,
            ),
            gradient: create_gradient_pipeline(
                &device,
                &texture_vs,
                &gradient_fs,
                msaa_sample_count,
                &vertex_buffers_description,
            ),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn create_pipeline_descriptor<'a>(
    label: Option<&'a str>,
    vertex_shader: &'a wgpu::ShaderModule,
    fragment_shader: &'a wgpu::ShaderModule,
    pipeline_layout: &'a wgpu::PipelineLayout,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
    color_states: &'a [wgpu::ColorStateDescriptor],
    vertex_buffers_description: &'a [wgpu::VertexBufferDescriptor<'a>],
    msaa_sample_count: u32,
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vertex_shader,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fragment_shader,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            polygon_mode: Default::default(),
            clamp_depth: false,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states,
        depth_stencil_state,
        sample_count: msaa_sample_count,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: vertex_buffers_description,
        },
    }
}

fn create_color_pipelines(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_description: &[wgpu::VertexBufferDescriptor<'_>],
) -> ShapePipeline {
    let bind_layout_label = create_debug_label!("Color shape bind group");
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: bind_layout_label.as_deref(),
    });

    let pipeline_layout_label = create_debug_label!("Color shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[&bind_layout],
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        },
    };

    ShapePipeline {
        mask_pipelines,
        bind_layout,
    }
}

fn create_bitmap_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_description: &[wgpu::VertexBufferDescriptor<'_>],
) -> ShapePipeline {
    let bind_layout_label = create_debug_label!("Bitmap shape bind group");
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
        ],
        label: bind_layout_label.as_deref(),
    });

    let pipeline_layout_label = create_debug_label!("Bitmap shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[&bind_layout],
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Equal,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        }
    };

    ShapePipeline {
        mask_pipelines,
        bind_layout,
    }
}

fn create_gradient_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_description: &[wgpu::VertexBufferDescriptor<'_>],
) -> ShapePipeline {
    let bind_layout_label = create_debug_label!("Gradient shape bind group");
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    min_binding_size: None,
                    readonly: true,
                },
                count: None,
            },
        ],
        label: bind_layout_label.as_deref(),
    });

    let pipeline_layout_label = create_debug_label!("Gradient shape pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts: &[&bind_layout],
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Equal,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
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
                Some(wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil,
                }),
                &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask,
                }],
                vertex_buffers_description,
                msaa_sample_count,
            ))
        }
    };

    ShapePipeline {
        mask_pipelines,
        bind_layout,
    }
}

fn mask_render_state(state: MaskState) -> (wgpu::StencilStateDescriptor, wgpu::ColorWrite) {
    let (stencil_state, color_write) = match state {
        MaskState::NoMask => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrite::ALL,
        ),
        MaskState::DrawMaskStencil => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::IncrementClamp,
            },
            wgpu::ColorWrite::empty(),
        ),
        MaskState::DrawMaskedContent => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            },
            wgpu::ColorWrite::ALL,
        ),
        MaskState::ClearMaskStencil => (
            wgpu::StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::DecrementClamp,
            },
            wgpu::ColorWrite::empty(),
        ),
    };

    (
        wgpu::StencilStateDescriptor {
            front: stencil_state.clone(),
            back: stencil_state,
            read_mask: 0xff,
            write_mask: 0xff,
        },
        color_write,
    )
}
