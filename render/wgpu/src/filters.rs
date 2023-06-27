mod blur;
mod color_matrix;

use crate::filters::blur::BlurFilter;
use crate::filters::color_matrix::ColorMatrixFilter;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::vertex_attr_array;

pub struct Filters {
    pub blur: BlurFilter,
    pub color_matrix: ColorMatrixFilter,
}

impl Filters {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            blur: BlurFilter::new(device),
            color_matrix: ColorMatrixFilter::new(device),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    }];

pub fn create_filter_vertices(
    device: &wgpu::Device,
    source_texture: &wgpu::Texture,
    source_point: (u32, u32),
    source_size: (u32, u32),
) -> wgpu::Buffer {
    let source_width = source_texture.width() as f32;
    let source_height = source_texture.height() as f32;
    let left = source_point.0;
    let top = source_point.1;
    let right = left + source_size.0;
    let bottom = top + source_size.1;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: create_debug_label!("Filter vertices").as_deref(),
        contents: bytemuck::cast_slice(&[
            FilterVertex {
                position: [0.0, 0.0],
                uv: [left as f32 / source_width, top as f32 / source_height],
            },
            FilterVertex {
                position: [1.0, 0.0],
                uv: [right as f32 / source_width, top as f32 / source_height],
            },
            FilterVertex {
                position: [1.0, 1.0],
                uv: [right as f32 / source_width, bottom as f32 / source_height],
            },
            FilterVertex {
                position: [0.0, 1.0],
                uv: [left as f32 / source_width, bottom as f32 / source_height],
            },
        ]),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
