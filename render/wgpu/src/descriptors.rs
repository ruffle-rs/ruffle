use crate::shaders::Shaders;
use crate::{BitmapSamplers, Pipelines};

/// Contains data specific to a `RenderTarget`.
/// We cannot re-use this data in `with_offscreen_backend`
pub struct DescriptorsTargetData {
    // These fields are specific to our `RenderTarget`, and
    // cannot be re-used
    pub pipelines: Pipelines,
    pub msaa_sample_count: u32,
}

impl DescriptorsTargetData {
    #[allow(clippy::too_many_arguments)]
    fn new(
        device: &wgpu::Device,
        shaders: &Shaders,
        surface_format: wgpu::TextureFormat,
        bitmap_samplers: &BitmapSamplers,
        msaa_sample_count: u32,
        globals_layout: &wgpu::BindGroupLayout,
        uniform_buffers_layout: &wgpu::BindGroupLayout,
        bitmap_bind_layout: &wgpu::BindGroupLayout,
        gradient_bind_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // We want to render directly onto a linear render target to avoid any gamma correction.
        // If our surface is sRGB, render to a linear texture and than copy over to the surface.
        // Remove Srgb from texture format.
        let frame_buffer_format = match surface_format {
            wgpu::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Bc1RgbaUnormSrgb => wgpu::TextureFormat::Bc1RgbaUnorm,
            wgpu::TextureFormat::Bc2RgbaUnormSrgb => wgpu::TextureFormat::Bc2RgbaUnorm,
            wgpu::TextureFormat::Bc3RgbaUnormSrgb => wgpu::TextureFormat::Bc3RgbaUnorm,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb => wgpu::TextureFormat::Bc7RgbaUnorm,
            wgpu::TextureFormat::Etc2Rgb8UnormSrgb => wgpu::TextureFormat::Etc2Rgb8Unorm,
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => wgpu::TextureFormat::Etc2Rgb8A1Unorm,
            wgpu::TextureFormat::Etc2Rgba8UnormSrgb => wgpu::TextureFormat::Etc2Rgba8Unorm,
            wgpu::TextureFormat::Astc {
                block,
                channel: wgpu::AstcChannel::UnormSrgb,
            } => wgpu::TextureFormat::Astc {
                block,
                channel: wgpu::AstcChannel::Unorm,
            },
            _ => surface_format,
        };

        let pipelines = Pipelines::new(
            device,
            shaders,
            surface_format,
            frame_buffer_format,
            msaa_sample_count,
            bitmap_samplers.layout(),
            globals_layout,
            uniform_buffers_layout,
            bitmap_bind_layout,
            gradient_bind_layout,
        );

        DescriptorsTargetData {
            pipelines,
            msaa_sample_count,
        }
    }
}

pub struct Descriptors {
    pub device: wgpu::Device,
    pub info: wgpu::AdapterInfo,
    pub limits: wgpu::Limits,
    pub surface_format: wgpu::TextureFormat,
    pub frame_buffer_format: wgpu::TextureFormat,
    pub queue: wgpu::Queue,
    pub globals_layout: wgpu::BindGroupLayout,
    pub uniform_buffers_layout: wgpu::BindGroupLayout,
    pub bitmap_samplers: BitmapSamplers,
    pub msaa_sample_count: u32,
    pub onscreen: DescriptorsTargetData,
    pub offscreen: DescriptorsTargetData,
    pub bitmap_bind_layout: wgpu::BindGroupLayout,
    pub gradient_bind_layout: wgpu::BindGroupLayout,
    pub copy_srgb_bind_layout: wgpu::BindGroupLayout,
}

impl Descriptors {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        info: wgpu::AdapterInfo,
        surface_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
    ) -> Self {
        let limits = device.limits();
        let bitmap_samplers = BitmapSamplers::new(&device);
        let shaders = Shaders::new(&device);

        let uniform_buffer_layout_label = create_debug_label!("Uniform buffer bind group layout");
        let uniform_buffers_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // We want to render directly onto a linear render target to avoid any gamma correction.
        // If our surface is sRGB, render to a linear texture and than copy over to the surface.
        // Remove Srgb from texture format.
        let frame_buffer_format = match surface_format {
            wgpu::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Bc1RgbaUnormSrgb => wgpu::TextureFormat::Bc1RgbaUnorm,
            wgpu::TextureFormat::Bc2RgbaUnormSrgb => wgpu::TextureFormat::Bc2RgbaUnorm,
            wgpu::TextureFormat::Bc3RgbaUnormSrgb => wgpu::TextureFormat::Bc3RgbaUnorm,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb => wgpu::TextureFormat::Bc7RgbaUnorm,
            wgpu::TextureFormat::Etc2Rgb8UnormSrgb => wgpu::TextureFormat::Etc2Rgb8Unorm,
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => wgpu::TextureFormat::Etc2Rgb8A1Unorm,
            wgpu::TextureFormat::Etc2Rgba8UnormSrgb => wgpu::TextureFormat::Etc2Rgba8Unorm,
            wgpu::TextureFormat::Astc {
                block,
                channel: wgpu::AstcChannel::UnormSrgb,
            } => wgpu::TextureFormat::Astc {
                block,
                channel: wgpu::AstcChannel::Unorm,
            },
            _ => surface_format,
        };

        let globals_layout_label = create_debug_label!("Globals bind group layout");
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let onscreen = DescriptorsTargetData::new(
            &device,
            &shaders,
            surface_format,
            &bitmap_samplers,
            msaa_sample_count,
            &globals_layout,
            &uniform_buffers_layout,
            &bitmap_bind_layout,
            &gradient_bind_layout,
        );

        // FIXME - get MSAA working for `TextureTarget`
        let offscreen = DescriptorsTargetData::new(
            &device,
            &shaders,
            wgpu::TextureFormat::Rgba8Unorm,
            &bitmap_samplers,
            1,
            &globals_layout,
            &uniform_buffers_layout,
            &bitmap_bind_layout,
            &gradient_bind_layout,
        );

        Self {
            device,
            info,
            limits,
            surface_format,
            frame_buffer_format,
            queue,
            globals_layout,
            uniform_buffers_layout,
            bitmap_samplers,
            msaa_sample_count,
            onscreen,
            offscreen,
            bitmap_bind_layout,
            gradient_bind_layout,
            copy_srgb_bind_layout,
        }
    }
}
