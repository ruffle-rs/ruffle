use bytemuck::Pod;
use std::{marker::PhantomData, mem};
use wgpu::util::StagingBelt;

/// A simple chunked bump allacator for managing dynamic uniforms that change per-draw.
/// Each draw call may use `UniformBuffer::write_uniforms` can be used to queue
/// the upload of uniform data to the GPU.
pub struct UniformBuffer<'a, T: Pod> {
    buffers: &'a mut BufferStorage<T>,
    cur_block: usize,
    cur_offset: u32,
}

pub struct BufferStorage<T: Pod> {
    _phantom: PhantomData<T>,
    blocks: Vec<Block>,
    staging_belt: StagingBelt,
    aligned_uniforms_size: u32,
}

impl<T: Pod> BufferStorage<T> {
    /// The size of each block.
    /// Uniforms are copied into each block until it reaches capacity, at which point a new
    /// block will be allocated.
    pub const BLOCK_SIZE: u32 = 65536;

    /// The uniform data size for a single draw call.
    pub const UNIFORMS_SIZE: u64 = mem::size_of::<T>() as u64;

    pub fn new(uniform_alignment: u32) -> Self {
        // Calculate alignment of uniforms.
        let align_mask = uniform_alignment - 1;
        let aligned_uniforms_size = (Self::UNIFORMS_SIZE as u32 + align_mask) & !align_mask;
        Self {
            blocks: Vec::with_capacity(8),
            staging_belt: StagingBelt::new(u64::from(Self::BLOCK_SIZE) / 2),
            aligned_uniforms_size,
            _phantom: PhantomData,
        }
    }

    /// Adds a newly allocated buffer to the block list, and returns it.
    pub fn allocate_block(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        let buffer_label = create_debug_label!("Dynamic buffer");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: Self::BLOCK_SIZE.into(),
            mapped_at_creation: false,
        });

        let bind_group_label = create_debug_label!("Dynamic buffer bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: bind_group_label.as_deref(),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<T>() as u64),
                }),
            }],
        });

        self.blocks.push(Block { buffer, bind_group });
    }
}

impl<'a, T: Pod> UniformBuffer<'a, T> {
    /// Creates a new `UniformBuffer` with the given uniform layout.
    pub fn new(buffers: &'a mut BufferStorage<T>) -> Self {
        buffers.staging_belt.recall();
        Self {
            buffers,
            cur_block: 0,
            cur_offset: 0,
        }
    }

    /// Enqueue `data` for upload into the given command encoder, and set the bind group on `render_pass`
    /// to use the uniform data.
    pub fn write_uniforms<'b>(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        command_encoder: &mut wgpu::CommandEncoder,
        render_pass: &mut wgpu::RenderPass<'b>,
        bind_group_index: u32,
        data: &T,
    ) where
        'a: 'b,
    {
        // Allocate a new block if we've exceeded our capacity.
        if self.cur_block >= self.buffers.blocks.len() {
            self.buffers.allocate_block(device, layout);
        }
        let block = &self.buffers.blocks[self.cur_block];

        // Copy the data into the buffer via the staging belt.
        self.buffers
            .staging_belt
            .write_buffer(
                command_encoder,
                &block.buffer,
                self.cur_offset.into(),
                BufferStorage::<T>::UNIFORMS_SIZE.try_into().unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::cast_slice(std::slice::from_ref(data)));

        // Set the bind group to the final uniform location.
        render_pass.set_bind_group(
            bind_group_index,
            unsafe { mem::transmute::<_, &'a wgpu::BindGroup>(&block.bind_group) },
            &[self.cur_offset],
        );

        // Advance offset.
        self.cur_offset += self.buffers.aligned_uniforms_size;
        // Advance to next buffer if we are out of room in this buffer.
        if BufferStorage::<T>::BLOCK_SIZE - self.cur_offset < self.buffers.aligned_uniforms_size {
            self.cur_block += 1;
            self.cur_offset = 0;
        }
    }

    /// Should be called at the end of a frame.
    pub fn finish(self) {
        self.buffers.staging_belt.finish();
    }
}

/// A block of GPU memory that will contain our uniforms.
#[derive(Debug)]
struct Block {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
