//use super::utils::create_debug_label;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Globals {
    bind_group: wgpu::BindGroup,
    _buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalsUniform {
    view_matrix: [[f32; 4]; 4],
}

impl Globals {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        Self::new_with_offset(device, layout, 0, 0, viewport_width, viewport_height)
    }

    /// Create globals that map a sub-region of viewport space to NDC.
    /// Content at (offset_x, offset_y) maps to top-left of the render target.
    pub fn new_with_offset(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        offset_x: u32,
        offset_y: u32,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        let w = viewport_width as f32;
        let h = viewport_height as f32;
        let ox = offset_x as f32;
        let oy = offset_y as f32;
        let temp_label = create_debug_label!("Globals buffer");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: temp_label.as_deref(),
            contents: bytemuck::cast_slice(&[GlobalsUniform {
                view_matrix: [
                    [2.0 / w, 0.0, 0.0, 0.0],
                    [0.0, -2.0 / h, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-2.0 * ox / w - 1.0, 2.0 * oy / h + 1.0, 0.0, 1.0],
                ],
            }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_label = create_debug_label!("Globals bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: bind_group_label.as_deref(),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            bind_group,
            _buffer: buffer,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
