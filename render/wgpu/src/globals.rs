//use super::utils::create_debug_label;
use bytemuck::{Pod, Zeroable};
use ruffle_render::{matrix3d::Matrix3D, perspective_projection::PerspectiveProjection};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Globals {
    bind_group: wgpu::BindGroup,
    _buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalsUniform {
    global_matrix: [[f32; 4]; 4],
}

impl Globals {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        // TODO: Currently, only the fixed default (Default::default) PerspectiveProjection is globally used for all objects.
        // Should support .transform.perspectiveProjection of each display object.
        // The global_matrix should be renamed/devided when supporting them.

        // film coordinates <= camera coordinates <= world coordinates <= model coordinates
        //                  ^ projection_matrix   ^ view_matrix        ^ model matrix
        //                  <==================================>
        //                             global_matrix
        //                   := projection_matrix x view_matrix

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
            // TODO: Consider PerspectiveProjection.projectionCenter.

            let perspective_projection = PerspectiveProjection::default();

            let perspective_projection_matrix = perspective_projection.to_matrix3d(2.0);

            let move_origin_matrix = {
                // AS3 places Viewpoint at (0, 0, -focalLength) in its coordination system.
                // Move Viewpoint to the origin (0, 0, 0) in order to have the projection work.
                let mut matrix = Matrix3D::IDENTITY;
                let focal_length = perspective_projection.focal_length(2.0);
                matrix.set_tz(focal_length.into());
                matrix
            };

            perspective_projection_matrix * move_origin_matrix
        };

        let global_matrix = projection_matrix * view_matrix;

        let global_matrix = {
            let mut matrix = [[0.0; 4]; 4];
            #[allow(clippy::needless_range_loop)]
            for i in 0..4 {
                for j in 0..4 {
                    matrix[j][i] = global_matrix.raw_data[i + 4 * j] as f32;
                }
            }
            matrix
        };

        let temp_label = create_debug_label!("Globals buffer");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: temp_label.as_deref(),
            contents: bytemuck::cast_slice(&[GlobalsUniform { global_matrix }]),
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
