use crate::descriptors::Descriptors;
use crate::Transforms;
use std::mem;

const ESTIMATED_OBJECTS_PER_CHUNK: u64 = 200;

pub struct DynamicTransforms {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl DynamicTransforms {
    pub fn new(descriptors: &Descriptors) -> Self {
        let buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (mem::size_of::<Transforms>() as u64 * ESTIMATED_OBJECTS_PER_CHUNK)
                .min(descriptors.limits.max_uniform_buffer_binding_size as u64)
                .min(descriptors.limits.max_buffer_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &descriptors.bind_layouts.transforms,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(mem::size_of::<Transforms>() as u64),
                    }),
                }],
            });
        Self { buffer, bind_group }
    }
}
