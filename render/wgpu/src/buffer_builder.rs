use bytemuck::{AnyBitPattern, NoUninit};
use std::ops::Range;
use wgpu::util::DeviceExt;

pub struct BufferBuilder {
    inner: Vec<u8>,
    align_mask: usize,
    limit: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct BufferFull;

impl BufferBuilder {
    pub fn new_for_vertices(limits: &wgpu::Limits) -> Self {
        Self {
            inner: Vec::new(),
            align_mask: 0,
            limit: limits.max_buffer_size,
        }
    }

    pub fn new_for_uniform(limits: &wgpu::Limits) -> Self {
        Self {
            inner: Vec::new(),
            align_mask: if limits.min_uniform_buffer_offset_alignment > 0 {
                (limits.min_uniform_buffer_offset_alignment - 1) as usize
            } else {
                0
            },
            limit: limits
                .max_buffer_size
                .min(limits.max_uniform_buffer_binding_size as u64),
        }
    }

    pub fn add<T: NoUninit + AnyBitPattern>(
        &mut self,
        value: &[T],
    ) -> Result<Range<wgpu::BufferAddress>, BufferFull> {
        let start_pos = if !self.inner.is_empty() {
            if self.align_mask > 0 {
                // Pad the internal buffer to match alignment requirements
                // Pad on creation so that we don't wastefully pad the end of the buffer
                (self.inner.len() + self.align_mask) & !self.align_mask
            } else {
                self.inner.len()
            }
        } else {
            0
        };

        if (start_pos + value.len()) as u64 > self.limit {
            return Err(BufferFull);
        }

        if start_pos > 0 && self.align_mask > 0 {
            self.inner.resize(start_pos, 0);
        }

        self.inner.extend_from_slice(bytemuck::cast_slice(value));
        Ok((start_pos as wgpu::BufferAddress)..(self.inner.len() as wgpu::BufferAddress))
    }

    pub fn finish(
        self,
        device: &wgpu::Device,
        label: Option<String>,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label.as_deref(),
            contents: &self.inner,
            usage,
        })
    }
}
