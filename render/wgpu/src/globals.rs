//use super::utils::create_debug_label;
use bytemuck::{Pod, Zeroable};
use ruffle_render::matrix3d::Matrix3D;
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
        let view_matrix = Matrix3D {
            raw_data: [
                1.0 / (viewport_width as f64 / 2.0),
                0.0,
                0.0,
                0.0,
                0.0,
                -1.0 / (viewport_height as f64 / 2.0),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0 / (viewport_width as f64 / 2.0),
                0.0,
                -1.0,
                1.0,
                0.0,
                1.0,
            ],
        };

        let projection_matrix = {
            // TODO: Here, just the fixed default value is used.
            // This should support variable values derived for each display object.
            let field_of_view = 55.0;

            let focal_length = {
                let deg2rad = f64::acos(-1.0) / 180.0;
                let rad = field_of_view / 2.0 * deg2rad;
                f64::cos(rad) / f64::sin(rad)
            };

            let perspective_projection = {
                let mut m = Matrix3D::IDENTITY;
                m.raw_data[0] = focal_length;
                m.raw_data[5] = focal_length;
                m.raw_data[10] = focal_length;
                m.raw_data[11] = 1.0;
                m.raw_data[14] = 0.0;
                m.raw_data[15] = 0.0;
                m
            };
            let move_coord = {
                // AS3 places Viewpoint at (0, 0, -focalLength).

                let mut m = Matrix3D::IDENTITY;
                m.raw_data[14] = focal_length;
                // TODO: Consider PerspectiveProjection.projectionCenter.
                m
            };
            let move_coord_back = {
                let mut m = Matrix3D::IDENTITY;
                m.raw_data[10] = 0.0;
                // TODO: Consider PerspectiveProjection.projectionCenter.
                m
            };
            move_coord_back * perspective_projection * move_coord
        };

        let matrix = projection_matrix * view_matrix;
        let matrix = {
            let mut m = [[0.0; 4]; 4];
            #[allow(clippy::needless_range_loop)]
            for i in 0..4 {
                for j in 0..4 {
                    m[j][i] = matrix.raw_data[i + 4 * j] as f32;
                }
            }
            m
        };

        let temp_label = create_debug_label!("Globals buffer");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: temp_label.as_deref(),
            contents: bytemuck::cast_slice(&[GlobalsUniform {
                view_matrix: matrix,
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
