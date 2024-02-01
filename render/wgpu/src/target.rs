use crate::buffer_pool::PoolEntry;
use crate::utils::BufferDimensions;
use crate::Error;
use ruffle_render::bitmap::PixelRegion;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tracing::instrument;

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
    ) -> wgpu::SubmissionIndex;
}

#[derive(Debug)]
pub struct SwapChainTarget {
    window_surface: wgpu::Surface<'static>,
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
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        (width, height): (u32, u32),
        device: &wgpu::Device,
    ) -> Self {
        // Ideally we want to use an RGBA non-sRGB surface format, because Flash colors and
        // blending are done in sRGB space -- we don't want the GPU to adjust the colors.
        // Some platforms may only support an sRGB surface, in which case we will draw to an
        // intermediate linear buffer and then copy to the sRGB surface.
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .or_else(|| capabilities.formats.first())
            .copied()
            // No surface (rendering to texture), default to linear RBGA.
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![format],
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

    #[instrument(level = "debug", skip_all)]
    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    ) -> wgpu::SubmissionIndex {
        let index = queue.submit(command_buffers);
        frame.texture.present();
        index
    }
}

#[derive(Debug)]
pub enum MaybeOwnedBuffer {
    Borrowed(PoolEntry<wgpu::Buffer, BufferDimensions>, BufferDimensions),
    Owned(wgpu::Buffer, BufferDimensions),
}

impl MaybeOwnedBuffer {
    pub fn inner(&self) -> (&wgpu::Buffer, &BufferDimensions) {
        match &self {
            MaybeOwnedBuffer::Borrowed(entry, dimensions) => ((*entry).deref(), dimensions),
            MaybeOwnedBuffer::Owned(buffer, dimensions) => (buffer, dimensions),
        }
    }
}

#[derive(Debug)]
pub struct TextureBufferInfo {
    pub buffer: MaybeOwnedBuffer,
    pub copy_area: PixelRegion,
}

#[derive(Debug)]
pub struct TextureTarget {
    pub size: wgpu::Extent3d,
    pub texture: Arc<wgpu::Texture>,
    pub format: wgpu::TextureFormat,
    pub buffer: Option<TextureBufferInfo>,
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
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let buffer_dimensions = BufferDimensions::new(size.0 as usize, size.1 as usize, format);
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture_label = create_debug_label!("Render target texture");
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: texture_label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            view_formats: &[format],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let buffer_label = create_debug_label!("Render target buffer");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size: (buffer_dimensions.padded_bytes_per_row as u64 * buffer_dimensions.height as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self {
            size,
            texture: Arc::new(texture),
            format,
            buffer: Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Owned(buffer, buffer_dimensions),
                copy_area: PixelRegion::for_whole_size(size.width, size.height),
            }),
        })
    }

    pub fn get_texture(&self) -> Arc<wgpu::Texture> {
        self.texture.clone()
    }

    pub fn take_buffer(self) -> Option<TextureBufferInfo> {
        self.buffer
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

    #[instrument(level = "debug", skip_all)]
    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        _frame: Self::Frame,
    ) -> wgpu::SubmissionIndex {
        if let Some(TextureBufferInfo { buffer, copy_area }) = &self.buffer {
            let label = create_debug_label!("Render target transfer encoder");
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: label.as_deref(),
            });
            let (buffer, dimensions) = buffer.inner();
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: copy_area.x_min,
                        y: copy_area.y_min,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(dimensions.padded_bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width: copy_area.width(),
                    height: copy_area.height(),
                    depth_or_array_layers: 1,
                },
            );
            queue.submit(command_buffers.into_iter().chain(Some(encoder.finish())))
        } else {
            queue.submit(command_buffers)
        }
    }
}
