use crate::layouts::BindLayouts;
use crate::shaders::Shaders;
use crate::{MaskState, Vertex};
use enum_map::{Enum, EnumMap};
use swf::BlendMode;
use wgpu::vertex_attr_array;

pub const VERTEX_BUFFERS_DESCRIPTION: [wgpu::VertexBufferLayout; 1] = [wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ],
}];

#[derive(Debug)]
pub struct ShapePipeline {
    pub pipelines: EnumMap<MaskState, wgpu::RenderPipeline>,
    depthless: wgpu::RenderPipeline,
}

#[derive(Enum, Debug)]
pub enum TrivialBlend {
    Normal,
}

#[derive(Enum, Debug)]
pub enum ComplexBlend {
    Multiply,
    Screen,
    Lighten,
    Darken,
    Difference,
    Add,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
}

impl ComplexBlend {
    pub fn id(&self) -> i32 {
        match self {
            ComplexBlend::Multiply => 1,
            ComplexBlend::Screen => 2,
            ComplexBlend::Lighten => 3,
            ComplexBlend::Darken => 4,
            ComplexBlend::Difference => 5,
            ComplexBlend::Add => 6,
            ComplexBlend::Subtract => 7,
            ComplexBlend::Invert => 8,
            ComplexBlend::Alpha => 9,
            ComplexBlend::Erase => 10,
            ComplexBlend::Overlay => 11,
            ComplexBlend::HardLight => 12,
        }
    }
}

#[derive(Debug)]
pub enum BlendType {
    Trivial(TrivialBlend),
    Complex(ComplexBlend),
}

impl BlendType {
    pub fn from(mode: BlendMode) -> BlendType {
        match mode {
            BlendMode::Normal => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Layer => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Multiply => BlendType::Complex(ComplexBlend::Multiply),
            BlendMode::Screen => BlendType::Complex(ComplexBlend::Screen),
            BlendMode::Lighten => BlendType::Complex(ComplexBlend::Lighten),
            BlendMode::Darken => BlendType::Complex(ComplexBlend::Darken),
            BlendMode::Difference => BlendType::Complex(ComplexBlend::Difference),
            BlendMode::Add => BlendType::Complex(ComplexBlend::Add),
            BlendMode::Subtract => BlendType::Complex(ComplexBlend::Subtract),
            BlendMode::Invert => BlendType::Complex(ComplexBlend::Invert),
            BlendMode::Alpha => BlendType::Complex(ComplexBlend::Alpha),
            BlendMode::Erase => BlendType::Complex(ComplexBlend::Erase),
            BlendMode::Overlay => BlendType::Complex(ComplexBlend::Overlay),
            BlendMode::HardLight => BlendType::Complex(ComplexBlend::HardLight),
        }
    }
}

#[derive(Debug)]
pub struct Pipelines {
    pub color: ShapePipeline,
    pub bitmap: EnumMap<TrivialBlend, ShapePipeline>,
    pub gradient: ShapePipeline,
    pub complex_blend: ShapePipeline,
}

impl ShapePipeline {
    pub fn pipeline_for(&self, mask_state: MaskState) -> &wgpu::RenderPipeline {
        &self.pipelines[mask_state]
    }

    pub fn depthless_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.depthless
    }

    /// Builds of a nested `EnumMap` that maps a `MaskState` to
    /// a `RenderPipeline`. The provided callback is used to construct the `RenderPipeline`
    /// for each possible `MaskState`.
    fn build(
        depthless: wgpu::RenderPipeline,
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
            depthless,
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
        let color_pipelines = create_shape_pipeline(
            "Color",
            device,
            format,
            &shaders.color_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[&bind_layouts.globals, &bind_layouts.transforms],
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        );

        let gradient_pipelines = create_shape_pipeline(
            "Gradient",
            device,
            format,
            &shaders.gradient_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.gradient,
            ],
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        );

        let complex_blend_pipeline = create_shape_pipeline(
            "Complex Blend",
            device,
            format,
            &shaders.blend_shader,
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION,
            &[
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.blend,
            ],
            wgpu::BlendState::REPLACE,
        );

        let bitmap_pipelines: [ShapePipeline; TrivialBlend::LENGTH] = (0..TrivialBlend::LENGTH)
            .map(|blend| {
                let blend = match TrivialBlend::from_usize(blend) {
                    TrivialBlend::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                };
                let name = format!("Bitmap ({blend:?})");
                create_shape_pipeline(
                    &name,
                    device,
                    format,
                    &shaders.bitmap_shader,
                    msaa_sample_count,
                    &VERTEX_BUFFERS_DESCRIPTION,
                    &[
                        &bind_layouts.globals,
                        &bind_layouts.transforms,
                        &bind_layouts.bitmap,
                    ],
                    blend,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            color: color_pipelines,
            bitmap: EnumMap::from_array(bitmap_pipelines),
            gradient: gradient_pipelines,
            complex_blend: complex_blend_pipeline,
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

#[allow(clippy::too_many_arguments)]
fn create_shape_pipeline(
    name: &str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    msaa_sample_count: u32,
    vertex_buffers_layout: &[wgpu::VertexBufferLayout<'_>],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    blend: wgpu::BlendState,
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("{} shape pipeline layout", name);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let mask_render_state = |mask_name, stencil_state, write_mask| {
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

    ShapePipeline::build(
        device.create_render_pipeline(&create_pipeline_descriptor(
            create_debug_label!("{} depthless pipeline", name).as_deref(),
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
