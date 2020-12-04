use crate::utils::BufferDimensions;
use futures::executor::block_on;
use image::buffer::ConvertBuffer;
use image::{Bgra, ImageBuffer, RgbaImage};
use std::fmt::Debug;

pub trait RenderTargetFrame: Debug {
    fn view(&self) -> &wgpu::TextureView;
}

pub trait RenderTarget: Debug + 'static {
    type Frame: RenderTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    fn format(&self) -> wgpu::TextureFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SwapChainError>;

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
    );
}

#[derive(Debug)]
pub struct SwapChainTarget {
    window_surface: wgpu::Surface,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

#[derive(Debug)]
pub struct SwapChainTargetFrame(wgpu::SwapChainFrame);

impl RenderTargetFrame for SwapChainTargetFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.0.output.view
    }
}

impl SwapChainTarget {
    pub fn new(surface: wgpu::Surface, size: (u32, u32), device: &wgpu::Device) -> Self {
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
        Self {
            window_surface: surface,
            swap_chain_desc,
            swap_chain,
        }
    }
}

impl RenderTarget for SwapChainTarget {
    type Frame = SwapChainTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = device.create_swap_chain(&self.window_surface, &self.swap_chain_desc);
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.swap_chain_desc.format
    }

    fn width(&self) -> u32 {
        self.swap_chain_desc.width
    }

    fn height(&self) -> u32 {
        self.swap_chain_desc.height
    }

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SwapChainError> {
        self.swap_chain
            .get_current_frame()
            .map(SwapChainTargetFrame)
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
    ) {
        queue.submit(command_buffers);
    }
}

#[derive(Debug)]
pub struct TextureTarget {
    size: wgpu::Extent3d,
    texture: wgpu::Texture,
    format: wgpu::TextureFormat,
    buffer: wgpu::Buffer,
    buffer_dimensions: BufferDimensions,
}

#[derive(Debug)]
pub struct TextureTargetFrame(wgpu::TextureView);

type BgraImage = ImageBuffer<Bgra<u8>, Vec<u8>>;

impl RenderTargetFrame for TextureTargetFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.0
    }
}

impl TextureTarget {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let buffer_dimensions = BufferDimensions::new(size.0 as usize, size.1 as usize);
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };
        let texture_label = create_debug_label!("Render target texture");
        let format = wgpu::TextureFormat::Bgra8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: texture_label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
        });
        let buffer_label = create_debug_label!("Render target buffer");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            size,
            texture,
            format,
            buffer,
            buffer_dimensions,
        }
    }

    pub fn capture(&self, device: &wgpu::Device) -> Option<RgbaImage> {
        let buffer_future = self.buffer.slice(..).map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);
        match block_on(buffer_future) {
            Ok(()) => {
                let map = self.buffer.slice(..).get_mapped_range();
                let mut buffer = Vec::with_capacity(
                    self.buffer_dimensions.height * self.buffer_dimensions.unpadded_bytes_per_row,
                );

                for chunk in map.chunks(self.buffer_dimensions.padded_bytes_per_row) {
                    buffer
                        .extend_from_slice(&chunk[..self.buffer_dimensions.unpadded_bytes_per_row]);
                }

                let bgra = BgraImage::from_raw(self.size.width, self.size.height, buffer);
                bgra.map(|image| image.convert())
            }
            Err(e) => {
                log::error!("Unknown error reading capture buffer: {:?}", e);
                None
            }
        }
    }
}

impl RenderTarget for TextureTarget {
    type Frame = TextureTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.size.width = width;
        self.size.height = height;

        let label = create_debug_label!("Render target texture");
        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size: self.size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
        });

        let buffer_label = create_debug_label!("Render target buffer");
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size: width as u64 * height as u64 * 4,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            mapped_at_creation: false,
        });
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn width(&self) -> u32 {
        self.size.width
    }

    fn height(&self) -> u32 {
        self.size.height
    }

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SwapChainError> {
        Ok(TextureTargetFrame(
            self.texture.create_view(&Default::default()),
        ))
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
    ) {
        let label = create_debug_label!("Render target transfer encoder");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: label.as_deref(),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &self.buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: self.buffer_dimensions.padded_bytes_per_row as u32,
                    rows_per_image: 0,
                },
            },
            self.size,
        );
        queue.submit(command_buffers.into_iter().chain(Some(encoder.finish())));
    }
}
