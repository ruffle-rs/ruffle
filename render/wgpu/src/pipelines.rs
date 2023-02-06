use crate::blend::{ComplexBlend, TrivialBlend};
use crate::descriptors::Descriptors;
use crate::layouts::BindLayouts;
use crate::shaders::Shaders;
use crate::{MaskState, PosColorVertex, PosVertex, PushConstants, Transforms};
use enum_map::{enum_map, Enum, EnumMap};
use once_cell::sync::OnceCell;
use ruffle_render::tessellator::GradientType;
use std::fmt::{Debug, Formatter};
use std::mem;
use swf::GradientSpread;
use wgpu::vertex_attr_array;

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

struct ShaderFn(Box<dyn Fn(&Shaders) -> &wgpu::ShaderModule + Sync + Send>);

impl Debug for ShaderFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShaderFn")
    }
}

#[derive(Debug)]
pub struct ShapePipeline {
    with_depth: EnumMap<MaskState, OnceCell<wgpu::RenderPipeline>>,
    depthless: OnceCell<wgpu::RenderPipeline>,
    layout: wgpu::PipelineLayout,
    shader_fn: ShaderFn,
    vertex_buffers: &'static [wgpu::VertexBufferLayout<'static>],
    color_format: wgpu::TextureFormat,
    sample_count: u32,
    blend: wgpu::BlendState,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color: ShapePipeline,
    pub bitmap: EnumMap<TrivialBlend, ShapePipeline>,
    pub gradients: EnumMap<GradientType, EnumMap<GradientSpread, ShapePipeline>>,
    pub complex_blends: EnumMap<ComplexBlend, ShapePipeline>,
    color_matrix_filter: OnceCell<wgpu::RenderPipeline>,
    blur_filter: OnceCell<wgpu::RenderPipeline>,
    format: wgpu::TextureFormat,
    sample_count: u32,
    full_push_constants: Vec<wgpu::PushConstantRange>,
}

impl ShapePipeline {
    pub fn pipeline_for(
        &self,
        mask_state: MaskState,
        descriptors: &Descriptors,
    ) -> &wgpu::RenderPipeline {
        self.with_depth[mask_state].get_or_init(|| {
            let shader_module = self.shader_fn.0(&descriptors.shaders);

            let (_, stencil_state, write_mask) = match mask_state {
                MaskState::NoMask => (
                    "no mask",
                    wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    wgpu::ColorWrites::ALL,
                ),
                MaskState::DrawMaskStencil => (
                    "draw mask stencil",
                    wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::IncrementClamp,
                    },
                    wgpu::ColorWrites::empty(),
                ),
                MaskState::DrawMaskedContent => (
                    "draw masked content",
                    wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    wgpu::ColorWrites::ALL,
                ),
                MaskState::ClearMaskStencil => (
                    "clear mask stencil",
                    wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::DecrementClamp,
                    },
                    wgpu::ColorWrites::empty(),
                ),
            };

            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&self.layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "main_vertex",
                        buffers: &self.vertex_buffers,
                    },
                    primitive: Default::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
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
                    multisample: wgpu::MultisampleState {
                        count: self.sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.color_format,
                            blend: Some(self.blend),
                            write_mask,
                        })],
                    }),
                    multiview: None,
                })
        })
    }

    pub fn depthless_pipeline(&self, descriptors: &Descriptors) -> &wgpu::RenderPipeline {
        self.depthless.get_or_init(|| {
            let shader_module = self.shader_fn.0(&descriptors.shaders);
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&self.layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "main_vertex",
                        buffers: &self.vertex_buffers,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: self.sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.color_format,
                            blend: Some(self.blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                })
        })
    }
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        msaa_sample_count: u32,
        bind_layouts: &BindLayouts,
    ) -> Self {
        let colort_bindings = if device.limits().max_push_constant_size > 0 {
            vec![&bind_layouts.globals]
        } else {
            vec![
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.color_transforms,
            ]
        };

        let full_push_constants = if device.limits().max_push_constant_size > 0 {
            vec![wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..mem::size_of::<PushConstants>() as u32,
            }]
        } else {
            vec![]
        };

        let partial_push_constants = if device.limits().max_push_constant_size > 0 {
            vec![wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..(mem::size_of::<Transforms>() as u32),
            }]
        } else {
            vec![]
        };

        let color_pipelines = create_shape_pipeline(
            "Color",
            device,
            format,
            Box::new(|shaders| &shaders.color_shader),
            msaa_sample_count,
            &VERTEX_BUFFERS_DESCRIPTION_COLOR,
            &colort_bindings,
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            &full_push_constants,
        );

        let gradient_bindings = if device.limits().max_push_constant_size > 0 {
            vec![&bind_layouts.globals, &bind_layouts.gradient]
        } else {
            vec![
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.color_transforms,
                &bind_layouts.gradient,
            ]
        };

        let gradient_pipelines = enum_map! {
            mode => enum_map! {
                spread =>
                    create_shape_pipeline(
                        &format!("Gradient - {mode:?} {spread:?}"),
                        device,
                        format,
                        Box::new(move |shaders| &shaders.gradient_shaders[mode][spread]),
                        msaa_sample_count,
                        &VERTEX_BUFFERS_DESCRIPTION_POS,
                        &gradient_bindings,
                        wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        &full_push_constants,
                    )

            }
        };

        let complex_blend_bindings = if device.limits().max_push_constant_size > 0 {
            vec![&bind_layouts.globals, &bind_layouts.blend]
        } else {
            vec![
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.blend,
            ]
        };

        let complex_blend_pipelines = enum_map! {
            blend => create_shape_pipeline(
                &format!("Complex Blend: {blend:?}"),
                device,
                format,
                Box::new(move |shaders| &shaders.blend_shaders[blend]),
                msaa_sample_count,
                &VERTEX_BUFFERS_DESCRIPTION_POS,
                &complex_blend_bindings,
                wgpu::BlendState::REPLACE,
                &partial_push_constants,
            )
        };

        let bitmap_blend_bindings = if device.limits().max_push_constant_size > 0 {
            vec![&bind_layouts.globals, &bind_layouts.bitmap]
        } else {
            vec![
                &bind_layouts.globals,
                &bind_layouts.transforms,
                &bind_layouts.color_transforms,
                &bind_layouts.bitmap,
            ]
        };

        let bitmap_pipelines: [ShapePipeline; TrivialBlend::LENGTH] = (0..TrivialBlend::LENGTH)
            .map(|blend| {
                let blend = TrivialBlend::from_usize(blend);
                let name = format!("Bitmap ({blend:?})");
                create_shape_pipeline(
                    &name,
                    device,
                    format,
                    Box::new(|shaders| &shaders.bitmap_shader),
                    msaa_sample_count,
                    &VERTEX_BUFFERS_DESCRIPTION_POS,
                    &bitmap_blend_bindings,
                    blend.blend_state(),
                    &full_push_constants,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            color: color_pipelines,
            bitmap: EnumMap::from_array(bitmap_pipelines),
            gradients: gradient_pipelines,
            complex_blends: complex_blend_pipelines,
            color_matrix_filter: OnceCell::new(),
            blur_filter: OnceCell::new(),
            format,
            sample_count: msaa_sample_count,
            full_push_constants,
        }
    }

    pub fn color_matrix_filter(&self, descriptors: &Descriptors) -> &wgpu::RenderPipeline {
        self.color_matrix_filter.get_or_init(|| {
            let bindings = if descriptors.limits.max_push_constant_size > 0 {
                vec![
                    &descriptors.bind_layouts.globals,
                    &descriptors.bind_layouts.bitmap,
                    &descriptors.bind_layouts.color_matrix_filter,
                ]
            } else {
                vec![
                    &descriptors.bind_layouts.globals,
                    &descriptors.bind_layouts.transforms,
                    &descriptors.bind_layouts.color_transforms,
                    &descriptors.bind_layouts.bitmap,
                    &descriptors.bind_layouts.color_matrix_filter,
                ]
            };
            let layout =
                descriptors
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bindings,
                        push_constant_ranges: &self.full_push_constants,
                    });
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.color_matrix_filter,
                        entry_point: "main_vertex",
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_POS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: self.sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &descriptors.shaders.color_matrix_filter,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                })
        })
    }

    pub fn blur_filter(&self, descriptors: &Descriptors) -> &wgpu::RenderPipeline {
        self.blur_filter.get_or_init(|| {
            let bindings = if descriptors.limits.max_push_constant_size > 0 {
                vec![
                    &descriptors.bind_layouts.globals,
                    &descriptors.bind_layouts.bitmap,
                    &descriptors.bind_layouts.blur_filter,
                ]
            } else {
                vec![
                    &descriptors.bind_layouts.globals,
                    &descriptors.bind_layouts.transforms,
                    &descriptors.bind_layouts.color_transforms,
                    &descriptors.bind_layouts.bitmap,
                    &descriptors.bind_layouts.blur_filter,
                ]
            };
            let layout =
                descriptors
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bindings,
                        push_constant_ranges: &self.full_push_constants,
                    });
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.blur_filter,
                        entry_point: "main_vertex",
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_POS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: self.sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &descriptors.shaders.blur_filter,
                        entry_point: "main_fragment",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                })
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn create_shape_pipeline(
    name: &str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader_fn: Box<dyn Fn(&Shaders) -> &wgpu::ShaderModule + Sync + Send>,
    msaa_sample_count: u32,
    vertex_buffers_layout: &'static [wgpu::VertexBufferLayout<'static>],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    blend: wgpu::BlendState,
    push_constant_ranges: &[wgpu::PushConstantRange],
) -> ShapePipeline {
    let pipeline_layout_label = create_debug_label!("{} shape pipeline layout", name);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: pipeline_layout_label.as_deref(),
        bind_group_layouts,
        push_constant_ranges,
    });

    ShapePipeline {
        with_depth: Default::default(),
        depthless: Default::default(),
        layout: pipeline_layout,
        shader_fn: ShaderFn(shader_fn),
        vertex_buffers: vertex_buffers_layout,
        color_format: format,
        sample_count: msaa_sample_count,
        blend,
    }
}
