use crate::{Error, GPUVertex};
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct ShapePipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_layout: wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color: ShapePipeline,
    pub bitmap: ShapePipeline,
    pub gradient: ShapePipeline,
}

impl Pipelines {
    pub fn new(device: &wgpu::Device) -> Result<Self, Error> {
        let depth_stencil_state = Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        });

        let color_vs_bytes = include_bytes!("../shaders/color.vert.spv");
        let color_vs = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(
            &color_vs_bytes[..],
        ))?);
        let color_fs_bytes = include_bytes!("../shaders/color.frag.spv");
        let color_fs = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(
            &color_fs_bytes[..],
        ))?);
        let texture_vs_bytes = include_bytes!("../shaders/texture.vert.spv");
        let texture_vs = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(
            &texture_vs_bytes[..],
        ))?);
        let gradient_fs_bytes = include_bytes!("../shaders/gradient.frag.spv");
        let gradient_fs = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(
            &gradient_fs_bytes[..],
        ))?);
        let bitmap_fs_bytes = include_bytes!("../shaders/bitmap.frag.spv");
        let bitmap_fs = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(
            &bitmap_fs_bytes[..],
        ))?);

        let (color_bind_layout, color_pipeline) =
            create_color_pipeline(&device, &color_vs, &color_fs, depth_stencil_state.clone());

        let (bitmap_bind_layout, bitmap_pipeline) = create_bitmap_pipeline(
            &device,
            &texture_vs,
            &bitmap_fs,
            depth_stencil_state.clone(),
        );

        let (gradient_bind_layout, gradient_pipeline) =
            create_gradient_pipeline(&device, &texture_vs, &gradient_fs, depth_stencil_state);

        Ok(Self {
            color: ShapePipeline {
                pipeline: color_pipeline,
                bind_layout: color_bind_layout,
            },
            bitmap: ShapePipeline {
                pipeline: bitmap_pipeline,
                bind_layout: bitmap_bind_layout,
            },
            gradient: ShapePipeline {
                pipeline: gradient_pipeline,
                bind_layout: gradient_bind_layout,
            },
        })
    }
}

fn create_pipeline_descriptor<'a>(
    vertex_shader: &'a wgpu::ShaderModule,
    fragment_shader: &'a wgpu::ShaderModule,
    pipeline_layout: &'a wgpu::PipelineLayout,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
    color_states: &'a [wgpu::ColorStateDescriptor],
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
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
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states,
        depth_stencil_state,
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<GPUVertex>() as u64,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &vertex_attr_array![
                    0 => Float2,
                    1 => Float4
                ],
            }],
        },
    }
}

fn create_color_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
) -> (wgpu::BindGroupLayout, wgpu::RenderPipeline) {
    let label = create_debug_label!("Color shape pipeline");
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: label.as_deref(),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let pipeline_descriptor = create_pipeline_descriptor(
        vertex_shader,
        fragment_shader,
        &pipeline_layout,
        depth_stencil_state,
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
            write_mask: wgpu::ColorWrite::ALL,
        }],
    );

    (
        bind_group_layout,
        device.create_render_pipeline(&pipeline_descriptor),
    )
}

fn create_bitmap_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
) -> (wgpu::BindGroupLayout, wgpu::RenderPipeline) {
    let label = create_debug_label!("Bitmap shape pipeline");
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
            },
        ],
        label: label.as_deref(),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let pipeline_descriptor = create_pipeline_descriptor(
        vertex_shader,
        fragment_shader,
        &pipeline_layout,
        depth_stencil_state,
        &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8Unorm,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            write_mask: wgpu::ColorWrite::ALL,
        }],
    );

    (
        bind_group_layout,
        device.create_render_pipeline(&pipeline_descriptor),
    )
}

fn create_gradient_pipeline(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
) -> (wgpu::BindGroupLayout, wgpu::RenderPipeline) {
    let label = create_debug_label!("Gradient shape pipeline");
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: label.as_deref(),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let pipeline_descriptor = create_pipeline_descriptor(
        vertex_shader,
        fragment_shader,
        &pipeline_layout,
        depth_stencil_state,
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
            write_mask: wgpu::ColorWrite::ALL,
        }],
    );

    (
        bind_group_layout,
        device.create_render_pipeline(&pipeline_descriptor),
    )
}
