use crate::descriptors::Descriptors;
use crate::{PosUvVertex, Transforms};
use std::mem;

const ESTIMATED_OBJECTS_PER_CHUNK: u64 = 200;

pub struct DynamicTransforms {
    pub buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn transforms_per_draw(limits: &wgpu::Limits) -> u64 {
    let maximum = limits.max_uniform_buffer_binding_size / size_of::<Transforms>() as u64;
    maximum.min(ESTIMATED_OBJECTS_PER_CHUNK)
}

impl DynamicTransforms {
    pub fn new(descriptors: &Descriptors) -> Self {
        let buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: transforms_per_draw(&descriptors.limits) * size_of::<Transforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (mem::size_of::<PosUvVertex>() as u64 * ESTIMATED_OBJECTS_PER_CHUNK)
                .min(descriptors.limits.max_buffer_size),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &descriptors.bind_layouts.transforms,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
        Self {
            buffer,
            bind_group,
            vertex_buffer,
        }
    }
}
