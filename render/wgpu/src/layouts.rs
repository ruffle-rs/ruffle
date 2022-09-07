#[derive(Debug)]
pub struct BindLayouts {
    pub globals: wgpu::BindGroupLayout,
    pub transforms: wgpu::BindGroupLayout,
    pub bitmap: wgpu::BindGroupLayout,
    pub gradient: wgpu::BindGroupLayout,
    pub copy_srgb: wgpu::BindGroupLayout,
    pub bitmap_sampler: wgpu::BindGroupLayout,
}

impl BindLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        let bitmap_sampler_layout_label = create_debug_label!("Sampler layout");
        let bitmap_sampler = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: bitmap_sampler_layout_label.as_deref(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }],
        });

        let uniform_buffer_layout_label = create_debug_label!("Uniform buffer bind group layout");
        let transforms = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: uniform_buffer_layout_label.as_deref(),
        });

        let globals_layout_label = create_debug_label!("Globals bind group layout");
        let globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: globals_layout_label.as_deref(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bitmap_bind_layout_label = create_debug_label!("Bitmap shape bind group layout");
        let bitmap = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let gradient_bind_layout_label = create_debug_label!("Gradient shape bind group");
        let gradient = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ty: if device.limits().max_storage_buffers_per_shader_stage > 0 {
                            wgpu::BufferBindingType::Storage { read_only: true }
                        } else {
                            wgpu::BufferBindingType::Uniform
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: gradient_bind_layout_label.as_deref(),
        });

        let copy_srgb = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        Self {
            globals,
            transforms,
            bitmap,
            gradient,
            copy_srgb,
            bitmap_sampler,
        }
    }
}
