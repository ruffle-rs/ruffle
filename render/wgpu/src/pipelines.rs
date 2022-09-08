use crate::layouts::BindLayouts;
use crate::shaders::Shaders;
use crate::{MaskState, Vertex};
use enum_map::{Enum, EnumMap};
use wgpu::vertex_attr_array;

const VERTEX_BUFFERS_DESCRIPTION: [wgpu::VertexBufferLayout; 1] = [wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ],
}];

#[derive(Debug, Enum, Copy, Clone)]
pub enum BlendMode {
    Normal,
    Add,
    Subtract,
}

// Use the GPU blend modes to roughly approximate Flash's blend modes.
// This should look reasonable for the most common cases, but full support requires
// rendering to an intermediate texture and custom shaders for the complex blend modes.
impl From<swf::BlendMode> for BlendMode {
    fn from(blend: swf::BlendMode) -> Self {
        match blend {
            swf::BlendMode::Normal => BlendMode::Normal,

            // dst + src
            swf::BlendMode::Add => BlendMode::Add,

            // dst - src
            swf::BlendMode::Subtract => BlendMode::Subtract,

            // Unsupported blend mode. Default to normal for now.
            _ => BlendMode::Normal,
        }
    }
}

impl BlendMode {
    pub fn blend_state(&self) -> wgpu::BlendState {
        match self {
            BlendMode::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            BlendMode::Add => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            BlendMode::Subtract => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
        }
    }
}

#[derive(Debug)]
pub struct ShapePipeline {
    pub pipelines: EnumMap<BlendMode, EnumMap<MaskState, wgpu::RenderPipeline>>,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color_pipelines: ShapePipeline,

    pub bitmap_pipelines: ShapePipeline,

    pub gradient_pipelines: ShapePipeline,

    pub copy_srgb_pipeline: wgpu::RenderPipeline,
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

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        shaders: &Shaders,
        surface_format: wgpu::TextureFormat,
        frame_buffer_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
        bind_layouts: &BindLayouts,
    ) -> Self {
        let color_pipelines = create_shape_pipeline(
            "Color",
            device,
            frame_buffer_format,
            &shaders.color_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[&bind_layouts.globals, &bind_layouts.transforms],
        );

        let bitmap_pipelines = create_shape_pipeline(
            "Bitmap",
            device,
            frame_buffer_format,
            &shaders.bitmap_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.bitmap,
                &bind_layouts.bitmap_sampler,
            ],
        );

        let gradient_pipelines = create_shape_pipeline(
            "Gradient",
            device,
            frame_buffer_format,
            &shaders.gradient_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.gradient,
            ],
        );

        let copy_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: create_debug_label!("Copy sRGB pipeline layout").as_deref(),
                bind_group_layouts: &[
                    &bind_layouts.globals,
                    &bind_layouts.transforms,
                    &bind_layouts.bitmap,
                    &bind_layouts.bitmap_sampler,
                ],
                push_constant_ranges: &[],
            });
        let copy_srgb_pipeline = device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("Copy sRGB pipeline").as_deref(),
            &shaders.copy_srgb_shader,
            &shaders.copy_srgb_shader,
            &copy_texture_pipeline_layout,
            None,
            &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: Default::default(),
            })],
            &VERTEX_BUFFERS_DESCRIPTION,
            1,
        ));

        Self {
            color_pipelines,
            bitmap_pipelines,
            gradient_pipelines,
            copy_srgb_pipeline,
        }
    }
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
                blend: Some(blend),
                write_mask,
            })],
            vertex_buffers_layout,
            msaa_sample_count,
        ))
    };

    ShapePipeline::build(|blend_mode, mask_state| {
        let blend = blend_mode.blend_state();
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
