use std::borrow::Cow;
use std::mem::size_of;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

macro_rules! create_debug_label {
    ($($arg:tt)*) => (
        if cfg!(feature = "render_debug_labels") {
            Some(format!($($arg)*))
        } else {
            None
        }
    )
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
#[derive(Debug)]
pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: NonZeroU32,
}

impl BufferDimensions {
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row =
            NonZeroU32::new((unpadded_bytes_per_row + padded_bytes_per_row_padding) as u32)
                .unwrap();

        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}
