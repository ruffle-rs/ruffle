use bytemuck::{AnyBitPattern, NoUninit};
use wgpu::util::DeviceExt;

pub struct BufferBuilder {
    inner: Vec<u8>,
    align_mask: usize,
}

impl BufferBuilder {
    pub fn new(alignment: usize) -> Self {
        Self {
            inner: Vec::new(),
            align_mask: alignment - 1,
        }
    }

    pub fn add<T: NoUninit + AnyBitPattern>(&mut self, value: T) -> wgpu::BufferAddress {
        if !self.inner.is_empty() {
            // Pad the internal buffer to match alignment requirements
            // Pad on creation so that we don't wastefully pad the end of the buffer
            let length = (self.inner.len() + self.align_mask) & !self.align_mask;
            self.inner.resize(length, 0);
        }

        let address = self.inner.len() as wgpu::BufferAddress;
        self.inner.extend_from_slice(bytemuck::cast_slice(&[value]));
        address
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
