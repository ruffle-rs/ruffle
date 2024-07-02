use crate::blend::{ComplexBlend, TrivialBlend};
use crate::layouts::BindLayouts;
use crate::shaders::Shaders;
use crate::{MaskState, PosColorVertex, PosVertex};
use enum_map::{enum_map, Enum, EnumMap};
use std::collections::HashMap;
use wgpu::{vertex_attr_array, BlendState, PrimitiveTopology};

pub const VERTEX_BUFFERS_DESCRIPTION_POS: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PosVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
        ],
    }];

pub const VERTEX_BUFFERS_DESCRIPTION_COLOR: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PosColorVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x4,
        ],
    }];

#[derive(Debug)]
pub struct ShapePipeline {
    pub pipelines: EnumMap<MaskState, wgpu::RenderPipeline>,
    stencilless: wgpu::RenderPipeline,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color: ShapePipeline,
    pub lines: ShapePipeline,
    /// Renders a bitmap without any blending, and does
    /// not write to the alpha channel. This is used for
    /// drawing a finished Stage3D buffer onto the background.
    pub bitmap_opaque: wgpu::RenderPipeline,
    /// Like `bitmap_opaque`, but with a no-op `DepthStencilState`.
    /// This is used when we're inside a `RenderPass` that is
    /// using a stencil buffer, but we don't want to write to it
    /// or use it in any way.
    pub bitmap_opaque_dummy_stencil: wgpu::RenderPipeline,
    pub bitmap: EnumMap<TrivialBlend, ShapePipeline>,
    pub gradients: ShapePipeline,
    pub complex_blends: EnumMap<ComplexBlend, ShapePipeline>,
}

impl ShapePipeline {
    pub fn pipeline_for(&self, mask_state: MaskState) -> &wgpu::RenderPipeline {
        &self.pipelines[mask_state]
    }

    pub fn stencilless_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.stencilless
    }

    /// Builds of a nested `EnumMap` that maps a `MaskState` to
    /// a `RenderPipeline`. The provided callback is used to construct the `RenderPipeline`
    /// for each possible `MaskState`.
    fn build(
        stencilless: wgpu::RenderPipeline,
        mut f: impl FnMut(MaskState) -> wgpu::RenderPipeline,
    ) -> Self {
        let mask_array: [wgpu::RenderPipeline; MaskState::LENGTH] = (0..MaskState::LENGTH)
            .map(|mask_enum| {
                let mask_state = MaskState::from_usize(mask_enum);
                f(mask_state)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        ShapePipeline {
            pipelines: EnumMap::from_array(mask_array),
            stencilless,
        }
    }
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        shaders: &Shaders,
        format: wgpu::TextureFormat,
        msaa_sample_count: u32,
        bind_layouts: &BindLayouts,
    ) -> Self {
        let colort_bindings = vec![&bind_layouts.globals, &bind_layouts.transforms];

        let color_pipelines = create_shape_pipeline(
            "Color",
            device,
            format,
            &shaders.color_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION_COLOR,
            &colort_bindings,
            BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            &[],
            PrimitiveTopology::TriangleList,
        );

        let lines_pipelines = create_shape_pipeline(
            "Lines",
            device,
            format,
            &shaders.color_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION_COLOR,
            &colort_bindings,
            BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            &[],
            PrimitiveTopology::LineStrip,
        );

        let gradient_bindings = vec![
            &bind_layouts.globals,
            &bind_layouts.transforms,
            &bind_layouts.gradient,
        ];

        let gradient_pipeline = create_shape_pipeline(
            "Gradient",
            device,
            format,
            &shaders.gradient_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION_POS,
            &gradient_bindings,
            BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            &[],
            PrimitiveTopology::TriangleList,
        );

        let complex_blend_bindings = vec![
            &bind_layouts.globals,
            &bind_layouts.transforms,
            &bind_layouts.blend,
        ];

        let complex_blend_pipelines = enum_map! {
            blend => create_shape_pipeline(
                &format!("Complex Blend: {blend:?}"),
                device,
                format,
                &shaders.blend_shaders[blend],
                msaa_sample_count,
                &VERTEX_BUFFERS_DESCRIPTION_POS,
                &complex_blend_bindings,
                BlendState::REPLACE,
                &[],
                PrimitiveTopology::TriangleList,
            )
        };

        let bitmap_blend_bindings = vec![
            &bind_layouts.globals,
            &bind_layouts.transforms,
            &bind_layouts.bitmap,
        ];

        let bitmap_pipelines: [ShapePipeline; TrivialBlend::LENGTH] = (0..TrivialBlend::LENGTH)
            .map(|blend| {
                let blend = TrivialBlend::from_usize(blend);
                let name = format!("Bitmap ({blend:?})");
                create_shape_pipeline(
                    &name,
                    device,
                    format,
                    &shaders.bitmap_shader,
                    msaa_sample_count,
                    &VERTEX_BUFFERS_DESCRIPTION_POS,
                    &bitmap_blend_bindings,
                    blend.blend_state(),
                    &[],
                    PrimitiveTopology::TriangleList,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let bitmap_opaque_pipeline_layout_label =
            create_debug_label!("Opaque bitmap pipeline layout");
        let bitmap_opaque_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: bitmap_opaque_pipeline_layout_label.as_deref(),
                bind_group_layouts: &bitmap_blend_bindings,
                push_constant_ranges: &[],
            });

        let bitmap_opaque = device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("Bitmap opaque copy").as_deref(),
            &shaders.bitmap_shader,
            &shaders.bitmap_shader,
            &bitmap_opaque_pipeline_layout,
            None,
            &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::COLOR,
            })],
            &VERTEX_BUFFERS_DESCRIPTION_POS,
            msaa_sample_count,
            &[("late_saturate".to_owned(), 1.0)].into(),
            PrimitiveTopology::TriangleList,
        ));

        let bitmap_opaque_dummy_depth = device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("Bitmap opaque copy").as_deref(),
            &shaders.bitmap_shader,
            &shaders.bitmap_shader,
            &bitmap_opaque_pipeline_layout,
            Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::COLOR,
            })],
            &VERTEX_BUFFERS_DESCRIPTION_POS,
            msaa_sample_count,
            &Default::default(),
            PrimitiveTopology::TriangleList,
        ));

        Self {
            color: color_pipelines,
            lines: lines_pipelines,
            bitmap: EnumMap::from_array(bitmap_pipelines),
            bitmap_opaque,
            bitmap_opaque_dummy_stencil: bitmap_opaque_dummy_depth,
            gradients: gradient_pipeline,
            complex_blends: complex_blend_pipelines,
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
    fragment_constants: &'a HashMap<String, f64>,
    primitive_topology: PrimitiveTopology,
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: "main_vertex",
            buffers: vertex_buffer_layout,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: "main_fragment",
            targets: color_target_state,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: fragment_constants,
                ..Default::default()
            },
        }),
        primitive: wgpu::PrimitiveState {
            topology: primitive_topology,
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

#[allow(clippy::too_many_arguments)]
fn create_shape_pipeline(
    name: &str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_layout: &[wgpu::VertexBufferLayout<'_>],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    blend: BlendState,
    push_constant_ranges: &[wgpu::PushConstantRange],
    primitive_topology: PrimitiveTopology,
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("{} shape pipeline layout", name);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts,
        push_constant_ranges,
    });

    let mask_render_state = |mask_name, stencil_state, write_mask| {
        device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("{} pipeline {}", name, mask_name).as_deref(),
            shader,
            shader,
            &pipeline_layout,
            Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
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
            &Default::default(),
            primitive_topology,
        ))
    };

    ShapePipeline::build(
        device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("{} stencilless pipeline", name).as_deref(),
            shader,
            shader,
            &pipeline_layout,
            None,
            &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            vertex_buffers_layout,
            msaa_sample_count,
            &Default::default(),
            primitive_topology,
        )),
        |mask_state| match mask_state {
            MaskState::NoMask => mask_render_state(
                "no mask",
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                wgpu::ColorWrites::ALL,
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
            ),
        },
    )
}
