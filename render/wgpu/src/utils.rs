use crate::buffer_pool::BufferDescription;
use crate::descriptors::Descriptors;
use crate::globals::Globals;
use crate::Transforms;
use ruffle_render::quality::StageQuality;
use std::borrow::Cow;
use std::mem::size_of;
use wgpu::util::DeviceExt;
use wgpu::CommandEncoder;

macro_rules! create_debug_label {
    ($($arg:tt)*) => (
        if cfg!(feature = "render_debug_labels") {
            Some(format!($($arg)*))
        } else {
            None
        }
    )
}

pub fn remove_srgb(format: wgpu::TextureFormat) -> wgpu::TextureFormat {
    match format {
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
        _ => format,
    }
}

pub fn format_list<'a>(values: &[&'a str], connector: &'a str) -> Cow<'a, str> {
    match values.len() {
        0 => Cow::Borrowed(""),
        1 => Cow::Borrowed(values[0]),
        _ => Cow::Owned(
            values[0..values.len() - 1].join(", ")
                + " "
                + connector
                + " "
                + values[values.len() - 1],
        ),
    }
}

pub fn get_backend_names(backends: wgpu::Backends) -> Vec<&'static str> {
    let mut names = Vec::new();

    if backends.contains(wgpu::Backends::VULKAN) {
        names.push("Vulkan");
    }
    if backends.contains(wgpu::Backends::DX12) {
        names.push("DirectX 12");
    }
    if backends.contains(wgpu::Backends::DX11) {
        names.push("DirectX 11");
    }
    if backends.contains(wgpu::Backends::METAL) {
        names.push("Metal");
    }
    if backends.contains(wgpu::Backends::GL) {
        names.push("Open GL");
    }
    if backends.contains(wgpu::Backends::BROWSER_WEBGPU) {
        names.push("Web GPU");
    }

    names
}

pub fn create_buffer_with_data(
    device: &wgpu::Device,
    data: &[u8],
    usage: wgpu::BufferUsages,
    label: Option<String>,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        usage,
        label: label.as_deref(),
        contents: data,
    })
}

// Based off wgpu example 'capture'
#[derive(Debug, Clone)]
pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: u32,
}

impl BufferDimensions {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = (unpadded_bytes_per_row + padded_bytes_per_row_padding) as u32;

        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn size(&self) -> u64 {
        self.padded_bytes_per_row as u64 * self.height as u64
    }
}

impl BufferDescription for BufferDimensions {
    type Cost = u64;

    fn cost_to_use(&self, other: &Self) -> Option<Self::Cost> {
        if self.size() <= other.size() {
            Some(other.size() - self.size())
        } else {
            None
        }
    }
}

pub fn capture_image<R, F: FnOnce(&[u8], u32) -> R>(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    dimensions: &BufferDimensions,
    index: Option<wgpu::SubmissionIndex>,
    with_rgba: F,
) -> R {
    let (sender, receiver) = std::sync::mpsc::channel();
    let buffer_slice = buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    device.poll(
        index
            .map(wgpu::Maintain::WaitForSubmissionIndex)
            .unwrap_or(wgpu::Maintain::Wait),
    );
    let _ = receiver.recv().expect("MPSC channel must not fail");
    let map = buffer_slice.get_mapped_range();
    let result = with_rgba(&map, dimensions.padded_bytes_per_row);
    drop(map);
    buffer.unmap();
    result
}

#[cfg(not(target_family = "wasm"))]
pub fn buffer_to_image(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    dimensions: &BufferDimensions,
    index: Option<wgpu::SubmissionIndex>,
    size: wgpu::Extent3d,
) -> image::RgbaImage {
    capture_image(device, buffer, dimensions, index, |rgba, _buffer_width| {
        let mut bytes = Vec::with_capacity(dimensions.height * dimensions.unpadded_bytes_per_row);

        for chunk in rgba.chunks(dimensions.padded_bytes_per_row as usize) {
            bytes.extend_from_slice(&chunk[..dimensions.unpadded_bytes_per_row]);
        }

        // The image copied from the GPU uses premultiplied alpha, so
        // convert to straight alpha if requested by the user.
        ruffle_render::utils::unmultiply_alpha_rgba(&mut bytes);

        image::RgbaImage::from_raw(size.width, size.height, bytes)
            .expect("Retrieved texture buffer must be a valid RgbaImage")
    })
}

pub fn supported_sample_count(
    adapter: &wgpu::Adapter,
    quality: StageQuality,
    format: wgpu::TextureFormat,
) -> u32 {
    let mut sample_count = quality.sample_count();
    let features = adapter.get_texture_format_features(format).flags;

    // Keep halving the sample count until we get one that's supported - or 1 (no multisampling)
    // It's not guaranteed that supporting 4x means supporting 2x, so there's no "max" option
    // And it's probably safer to round down than up, given it's a performance setting.
    while sample_count > 1 && !features.sample_count_supported(sample_count) {
        sample_count /= 2;
    }
    sample_count
}

#[allow(clippy::too_many_arguments)]
pub fn run_copy_pipeline(
    descriptors: &Descriptors,
    format: wgpu::TextureFormat,
    actual_surface_format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
    frame_view: &wgpu::TextureView,
    input: &wgpu::TextureView,
    whole_frame_bind_group: &wgpu::BindGroup,
    globals: &Globals,
    sample_count: u32,
    encoder: &mut CommandEncoder,
) {
    let copy_bind_group = descriptors
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &descriptors.bind_layouts.bitmap,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: descriptors.quad.texture_transforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(
                        descriptors.bitmap_samplers.get_sampler(false, false),
                    ),
                },
            ],
            label: create_debug_label!("Copy sRGB bind group").as_deref(),
        });

    let pipeline = if actual_surface_format == format {
        descriptors.copy_pipeline(format, sample_count)
    } else {
        descriptors.copy_srgb_pipeline(actual_surface_format, sample_count)
    };

    // We overwrite the pixels in the target texture (no blending at all),
    // so this doesn't matter.
    let load = wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT);

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: create_debug_label!("Copy back to render target").as_deref(),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame_view,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
            resolve_target: None,
        })],
        ..Default::default()
    });

    render_pass.set_pipeline(&pipeline);
    render_pass.set_bind_group(0, globals.bind_group(), &[]);

    if descriptors.limits.max_push_constant_size > 0 {
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::cast_slice(&[Transforms {
                world_matrix: [
                    [size.width as f32, 0.0, 0.0, 0.0],
                    [0.0, size.height as f32, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }]),
        );
        render_pass.set_bind_group(1, &copy_bind_group, &[]);
    } else {
        render_pass.set_bind_group(1, whole_frame_bind_group, &[0]);
        render_pass.set_bind_group(2, &copy_bind_group, &[]);
    }

    render_pass.set_vertex_buffer(0, descriptors.quad.vertices_pos.slice(..));
    render_pass.set_index_buffer(
        descriptors.quad.indices.slice(..),
        wgpu::IndexFormat::Uint32,
    );

    render_pass.draw_indexed(0..6, 0, 0..1);
    drop(render_pass);
}

#[derive(Debug)]
pub struct SampleCountMap<T> {
    one: T,
    two: T,
    four: T,
    eight: T,
    sixteen: T,
}

impl<T: Default> Default for SampleCountMap<T> {
    fn default() -> Self {
        SampleCountMap {
            one: Default::default(),
            two: Default::default(),
            four: Default::default(),
            eight: Default::default(),
            sixteen: Default::default(),
        }
    }
}

impl<T> SampleCountMap<T> {
    pub fn get(&self, sample_count: u32) -> &T {
        match sample_count {
            1 => &self.one,
            2 => &self.two,
            4 => &self.four,
            8 => &self.eight,
            16 => &self.sixteen,
            _ => unreachable!("Sample counts must be powers of two between 1..=16"),
        }
    }
}

impl<T> SampleCountMap<std::sync::OnceLock<T>> {
    pub fn get_or_init<F>(&self, sample_count: u32, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match sample_count {
            1 => self.one.get_or_init(init),
            2 => self.two.get_or_init(init),
            4 => self.four.get_or_init(init),
            8 => self.eight.get_or_init(init),
            16 => self.sixteen.get_or_init(init),
            _ => unreachable!("Sample counts must be powers of two between 1..=16"),
        }
    }
}
