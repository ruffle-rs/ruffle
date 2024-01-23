use crate::descriptors::Descriptors;
use crate::{ColorAdjustments, Transforms};
use std::marker::PhantomData;
use std::mem;

const ESTIMATED_OBJECTS_PER_CHUNK: u64 = 200;

pub struct DynamicTransforms {
    pub transform: Inner<Transforms>,
    pub color: Inner<ColorAdjustments>,
}

impl DynamicTransforms {
    pub fn new(descriptors: &Descriptors) -> Self {
        Self {
            transform: Inner::new(&descriptors.device, &descriptors.bind_layouts.transforms),
            color: Inner::new(
                &descriptors.device,
                &descriptors.bind_layouts.color_transforms,
            ),
        }
    }
}

pub struct Inner<T> {
    _phantom: PhantomData<T>,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<T> Inner<T> {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (mem::size_of::<T>() as u64 * ESTIMATED_OBJECTS_PER_CHUNK)
                .min(device.limits().max_uniform_buffer_binding_size as u64)
                .min(device.limits().max_buffer_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(mem::size_of::<T>() as u64),
                }),
            }],
        });
        Self {
            _phantom: PhantomData,
            buffer,
            bind_group,
        }
    }
}
