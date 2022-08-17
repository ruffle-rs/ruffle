use crate::{Error, MaskState, Vertex};
use enum_map::{Enum, EnumMap};
use swf::BlendMode;
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct ShapePipeline {
    pub pipelines: EnumMap<BlendMode, EnumMap<MaskState, wgpu::RenderPipeline>>,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color_pipelines: ShapePipeline,

    pub bitmap_pipelines: ShapePipeline,
    pub bitmap_layout: wgpu::BindGroupLayout,

    pub gradient_pipelines: ShapePipeline,
    pub gradient_layout: wgpu::BindGroupLayout,

    pub copy_srgb_pipeline: wgpu::RenderPipeline,
    pub copy_srgb_layout: wgpu::BindGroupLayout,
}

impl ShapePipeline {
    pub fn pipeline_for(
        &self,
        blend_mode: BlendMode,
        mask_state: MaskState,
    ) -> &wgpu::RenderPipeline {
        &self.pipelines[blend_mode][mask_state]
    }

    /// Builds of a nested `EnumMap` that maps a `BlendMode` and `MaskState` to
    /// a `RenderPipeline`. The provided callback is used to construct the `RenderPipeline`
    /// for each possible `(BlendMode, MaskState)` pair.
    fn build(mut f: impl FnMut(BlendMode, MaskState) -> wgpu::RenderPipeline) -> Self {
        let blend_array: [EnumMap<MaskState, wgpu::RenderPipeline>; BlendMode::LENGTH] = (0
            ..BlendMode::LENGTH)
            .map(|blend_enum| {
                let blend_mode = BlendMode::from_usize(blend_enum);
                let mask_array: [wgpu::RenderPipeline; MaskState::LENGTH] = (0..MaskState::LENGTH)
                    .map(|mask_enum| {
                        let mask_state = MaskState::from_usize(mask_enum);
                        f(blend_mode, mask_state)
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                EnumMap::from_array(mask_array)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        ShapePipeline {
            pipelines: EnumMap::from_array(blend_array),
        }
    }
}

fn blend_mode_to_state(mode: BlendMode) -> Option<wgpu::BlendState> {
    // Use the GPU blend modes to roughly approximate Flash's blend modes.
    // This should look reasonable for the most common cases, but full support requires
    // rendering to an intermediate texture and custom shaders for the complex blend modes.
    match mode {
        BlendMode::Normal => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),

        // TODO: Needs intermediate buffer.
        BlendMode::Layer => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),

        // dst * src
        BlendMode::Multiply => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // 1 - (1 - dst) * (1 - src)
        // TODO: Needs shader. Rendererd as additive for now.
        BlendMode::Screen => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // max(dst, src)
        BlendMode::Lighten => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Max,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // min(dst, src)
        BlendMode::Darken => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Min,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // abs(dst - src)
        // TODO: Needs shader. Rendererd as subtract for now.
        BlendMode::Difference => {
            Some(wgpu::BlendState {
                // Add src and dst RGB values together
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent::OVER,
            })
        }

        // dst + src
        BlendMode::Add => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // dst - src
        BlendMode::Subtract => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::ReverseSubtract,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // 1 - dst
        BlendMode::Invert => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusDst,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // TODO: Requires intermediate buffer.
        // dst.alpha = src.alpha
        // Parent display object needs to have Layer blend mode.
        BlendMode::Alpha => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),

        // TODO: Requires intermediate buffer.
        // dst.alpha = 1 - src.alpha
        // Parent display object needs to have Layer blend mode.
        BlendMode::Erase => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),

        // if src > .5 { 1 - (1 - dst) * (1 - src) } else { dst * src }
        // TODO: Needs shader, rendered as multiply for now.
        BlendMode::HardLight => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),

        // if dst > .5 { 1 - (1 - dst) * (1 - src) } else { dst * src }
        // TODO: Needs shader, rendered as multiply for now.
        BlendMode::Overlay => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }),
    }
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        frame_buffer_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
        sampler_layout: &wgpu::BindGroupLayout,
        globals_layout: &wgpu::BindGroupLayout,
        dynamic_uniforms_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, Error> {
        let color_shader = create_shader(device, "color", include_str!("../shaders/color.wgsl"));
        let bitmap_shader = create_shader(device, "bitmap", include_str!("../shaders/bitmap.wgsl"));
        let gradient_shader =
            create_shader(device, "gradient", include_str!("../shaders/gradient.wgsl"));
        let copy_srgb_shader = create_shader(
            device,
            "copy sRGB",
            include_str!("../shaders/copy_srgb.wgsl"),
        );

        let vertex_buffers_description = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Float32x4,
            ],
        }];

        let color_pipelines = create_shape_pipeline(
            "Color",
            device,
            frame_buffer_format,
            &color_shader,
            msaa_sample_count,
            &vertex_buffers_description,
            &[globals_layout, dynamic_uniforms_layout],
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

        let bitmap_pipelines = create_shape_pipeline(
            "Bitmap",
            device,
            frame_buffer_format,
            &bitmap_shader,
            msaa_sample_count,
            &vertex_buffers_description,
            &[
                globals_layout,
                dynamic_uniforms_layout,
                &bitmap_bind_layout,
                sampler_layout,
            ],
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

        let gradient_pipelines = create_shape_pipeline(
            "Gradient",
            device,
            frame_buffer_format,
            &gradient_shader,
            msaa_sample_count,
            &vertex_buffers_description,
            &[
                globals_layout,
                dynamic_uniforms_layout,
                &gradient_bind_layout,
            ],
        );

        let copy_srgb_bind_layout =
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
                label: create_debug_label!("Copy sRGB bind group layout").as_deref(),
            });
        let copy_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: create_debug_label!("Copy sRGB pipeline layout").as_deref(),
                bind_group_layouts: &[
                    globals_layout,
                    dynamic_uniforms_layout,
                    &bitmap_bind_layout,
                    sampler_layout,
                ],
                push_constant_ranges: &[],
            });
        let copy_srgb_pipeline = device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("Copy sRGB pipeline").as_deref(),
            &copy_srgb_shader,
            &copy_srgb_shader,
            &copy_texture_pipeline_layout,
            None,
            &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: Default::default(),
            })],
            &vertex_buffers_description,
            1,
        ));

        Ok(Self {
            color_pipelines,
            bitmap_pipelines,
            bitmap_layout: bitmap_bind_layout,
            gradient_pipelines,
            gradient_layout: gradient_bind_layout,
            copy_srgb_pipeline,
            copy_srgb_layout: copy_srgb_bind_layout,
        })
    }
}

/// Builds a `wgpu::ShaderModule` the given WGSL source in `src`.
///
/// The source is prepended with common code in `common.wgsl`, simulating a `#include` preprocessor.
/// We could possibly does this as an offline build step instead.
fn create_shader(
    device: &wgpu::Device,
    name: &'static str,
    src: &'static str,
) -> wgpu::ShaderModule {
    const COMMON_SRC: &str = include_str!("../shaders/common.wgsl");
    let src = [COMMON_SRC, src].concat();
    let label = create_debug_label!("Shader {}", name,);
    let desc = wgpu::ShaderModuleDescriptor {
        label: label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    };
    device.create_shader_module(desc)
}

#[allow(clippy::too_many_arguments)]
fn create_pipeline_descriptor<'a>(
    label: Option<&'a str>,
    vertex_shader: &'a wgpu::ShaderModule,
    fragment_shader: &'a wgpu::ShaderModule,
    pipeline_layout: &'a wgpu::PipelineLayout,
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    color_target_state: &'a [Option<wgpu::ColorTargetState>],
    vertex_buffer_layout: &'a [wgpu::VertexBufferLayout<'a>],
    msaa_sample_count: u32,
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: "main_vertex",
            buffers: vertex_buffer_layout,
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: "main_fragment",
            targets: color_target_state,
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
        depth_stencil: depth_stencil_state,
        multisample: wgpu::MultisampleState {
            count: msaa_sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    }
}

fn create_shape_pipeline(
    name: &'static str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_layout: &[wgpu::VertexBufferLayout<'_>],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("{} shape pipeline layout", name);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let mask_render_state = |mask_name, stencil_state, write_mask, blend| {
        device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("{} pipeline {}", name, mask_name).as_deref(),
            shader,
            shader,
            &pipeline_layout,
            Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: Default::default(),
            }),
            &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask,
            })],
            vertex_buffers_layout,
            msaa_sample_count,
        ))
    };

    ShapePipeline::build(|blend_mode, mask_state| {
        let blend = blend_mode_to_state(blend_mode);
        match mask_state {
            MaskState::NoMask => mask_render_state(
                "no mask",
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                wgpu::ColorWrites::ALL,
                blend,
            ),
            MaskState::DrawMaskStencil => mask_render_state(
                "draw mask stencil",
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::IncrementClamp,
                },
                wgpu::ColorWrites::empty(),
                blend,
            ),
            MaskState::DrawMaskedContent => mask_render_state(
                "draw masked content",
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                wgpu::ColorWrites::ALL,
                blend,
            ),
            MaskState::ClearMaskStencil => mask_render_state(
                "clear mask stencil",
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::DecrementClamp,
                },
                wgpu::ColorWrites::empty(),
                blend,
            ),
        }
    })
}
