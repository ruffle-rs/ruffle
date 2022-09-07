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
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bitmap_samplers: &BitmapSamplers,
        msaa_sample_count: u32,
        globals_layout: &wgpu::BindGroupLayout,
        uniform_buffers_layout: &wgpu::BindGroupLayout,
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
            &device,
            surface_format,
            frame_buffer_format,
            msaa_sample_count,
            bitmap_samplers.layout(),
            globals_layout,
            uniform_buffers_layout,
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

        let onscreen = DescriptorsTargetData::new(
            &device,
            surface_format,
            &bitmap_samplers,
            msaa_sample_count,
            &globals_layout,
            &uniform_buffers_layout,
        );

        // FIXME - get MSAA working for `TextureTarget`
        let offscreen = DescriptorsTargetData::new(
            &device,
            wgpu::TextureFormat::Rgba8Unorm,
            &bitmap_samplers,
            1,
            &globals_layout,
            &uniform_buffers_layout,
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
        }
    }
}
