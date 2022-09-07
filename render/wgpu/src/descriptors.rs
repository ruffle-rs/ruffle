use crate::layouts::BindLayouts;
use crate::shaders::Shaders;
use crate::{create_buffer_with_data, BitmapSamplers, Pipelines, TextureTransforms, Vertex};

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
        shaders: &Shaders,
        surface_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
        bind_layouts: &BindLayouts,
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
            bind_layouts,
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
    pub bitmap_samplers: BitmapSamplers,
    pub msaa_sample_count: u32,
    pub onscreen: DescriptorsTargetData,
    pub offscreen: DescriptorsTargetData,
    pub bind_layouts: BindLayouts,
    pub quad: Quad,
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
        let bind_layouts = BindLayouts::new(&device);
        let bitmap_samplers = BitmapSamplers::new(&device, &bind_layouts);
        let shaders = Shaders::new(&device);
        let quad = Quad::new(&device);

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

        let onscreen = DescriptorsTargetData::new(
            &device,
            &shaders,
            surface_format,
            msaa_sample_count,
            &bind_layouts,
        );

        let offscreen = DescriptorsTargetData::new(
            &device,
            &shaders,
            wgpu::TextureFormat::Rgba8Unorm,
            msaa_sample_count,
            &bind_layouts,
        );

        Self {
            device,
            info,
            limits,
            surface_format,
            frame_buffer_format,
            queue,
            bitmap_samplers,
            msaa_sample_count,
            onscreen,
            offscreen,
            bind_layouts,
            quad,
        }
    }
}

pub struct Quad {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub texture_transforms: wgpu::Buffer,
}

impl Quad {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = [
            Vertex {
                position: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vbo = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Quad vbo"),
        );

        let ibo = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&indices),
            wgpu::BufferUsages::INDEX,
            create_debug_label!("Quad ibo"),
        );

        let tex_transforms = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&[TextureTransforms {
                u_matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Quad tex transforms"),
        );

        Self {
            vertices: vbo,
            indices: ibo,
            texture_transforms: tex_transforms,
        }
    }
}
