use crate::utils::BufferDimensions;
use crate::Error;
use ruffle_render::utils::unmultiply_alpha_rgba;
use std::fmt::Debug;

pub trait RenderTargetFrame: Debug {
    fn into_view(self) -> wgpu::TextureView;

    fn view(&self) -> &wgpu::TextureView;
}

pub trait RenderTarget: Debug + 'static {
    type Frame: RenderTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    fn format(&self) -> wgpu::TextureFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError>;

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    );
}

#[derive(Debug)]
pub struct SwapChainTarget {
    window_surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

#[derive(Debug)]
pub struct SwapChainTargetFrame {
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl RenderTargetFrame for SwapChainTargetFrame {
    fn into_view(self) -> wgpu::TextureView {
        self.view
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl SwapChainTarget {
    pub fn new(
        surface: wgpu::Surface,
        adapter: &wgpu::Adapter,
        (width, height): (u32, u32),
        device: &wgpu::Device,
    ) -> Self {
        // Ideally we want to use an RGBA non-sRGB surface format, because Flash colors and
        // blending are done in sRGB space -- we don't want the GPU to adjust the colors.
        // Some platforms may only support an sRGB surface, in which case we will draw to an
        // intermediate linear buffer and then copy to the sRGB surface.
        let formats = surface.get_supported_formats(adapter);
        let format = formats
            .iter()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .or_else(|| formats.first())
            .copied()
            // No surface (rendering to texture), default to linear RBGA.
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(adapter)[0],
        };
        surface.configure(device, &surface_config);
        Self {
            surface_config,
            window_surface: surface,
        }
    }
}

impl RenderTarget for SwapChainTarget {
    type Frame = SwapChainTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.window_surface.configure(device, &self.surface_config);
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    fn width(&self) -> u32 {
        self.surface_config.width
    }

    fn height(&self) -> u32 {
        self.surface_config.height
    }

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError> {
        let texture = self.window_surface.get_current_texture()?;
        let view = texture.texture.create_view(&Default::default());
        Ok(SwapChainTargetFrame { texture, view })
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    ) {
        queue.submit(command_buffers);
        frame.texture.present();
    }
}

#[derive(Debug)]
pub struct TextureTarget {
    pub size: wgpu::Extent3d,
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    pub buffer: wgpu::Buffer,
    pub buffer_dimensions: BufferDimensions,
}

#[derive(Debug)]
pub struct TextureTargetFrame(wgpu::TextureView);

impl RenderTargetFrame for TextureTargetFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.0
    }

    fn into_view(self) -> wgpu::TextureView {
        self.0
    }
}

impl TextureTarget {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Result<Self, Error> {
        if size.0 > device.limits().max_texture_dimension_2d
            || size.1 > device.limits().max_texture_dimension_2d
            || size.0 < 1
            || size.1 < 1
        {
            return Err(format!(
                "Texture target cannot be smaller than 1 or larger than {}px on either dimension (requested {} x {})",
                device.limits().max_texture_dimension_2d,
                size.0,
                size.1
            )
            .into());
        }
        let buffer_dimensions = BufferDimensions::new(size.0 as usize, size.1 as usize);
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture_label = create_debug_label!("Render target texture");
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: texture_label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        });
        let buffer_label = create_debug_label!("Render target buffer");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size: (buffer_dimensions.padded_bytes_per_row.get() as u64
                * buffer_dimensions.height as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self {
            size,
            texture,
            format,
            buffer,
            buffer_dimensions,
        })
    }

    /// Captures the current contents of our texture buffer
    /// as an `RgbaImage`
    pub fn capture(
        &self,
        device: &wgpu::Device,
        premultiplied_alpha: bool,
    ) -> Option<image::RgbaImage> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        let result = receiver.recv().unwrap();
        match result {
            Ok(()) => {
                let map = buffer_slice.get_mapped_range();
                let mut buffer = Vec::with_capacity(
                    self.buffer_dimensions.height * self.buffer_dimensions.unpadded_bytes_per_row,
                );

                for chunk in map.chunks(self.buffer_dimensions.padded_bytes_per_row.get() as usize)
                {
                    buffer
                        .extend_from_slice(&chunk[..self.buffer_dimensions.unpadded_bytes_per_row]);
                }

                // The image copied from the GPU uses premultiplied alpha, so
                // convert to straight alpha if requested by the user.
                if !premultiplied_alpha {
                    unmultiply_alpha_rgba(&mut buffer);
                }

                let image = image::RgbaImage::from_raw(self.size.width, self.size.height, buffer);
                drop(map);
                self.buffer.unmap();
                image
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
        *self =
            TextureTarget::new(device, (width, height)).expect("Unable to resize texture target");
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

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError> {
        Ok(TextureTargetFrame(
            self.texture.create_view(&Default::default()),
        ))
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        _frame: Self::Frame,
    ) {
        let label = create_debug_label!("Render target transfer encoder");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: label.as_deref(),
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.buffer_dimensions.padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            self.size,
        );
        queue.submit(command_buffers.into_iter().chain(Some(encoder.finish())));
    }
}
