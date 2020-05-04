use raw_window_handle::HasRawWindowHandle;
use std::fmt::Debug;

pub trait RenderTargetFrame: Debug {
    fn view(&self) -> &wgpu::TextureView;
}

pub trait RenderTarget: Debug {
    type Frame: RenderTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    fn format(&self) -> wgpu::TextureFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::TimeOut>;
}

#[derive(Debug)]
pub struct SwapChainTarget {
    window_surface: wgpu::Surface,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

#[derive(Debug)]
pub struct SwapChainTargetFrame(wgpu::SwapChainOutput);

impl RenderTargetFrame for SwapChainTargetFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.0.view
    }
}

impl SwapChainTarget {
    pub fn new<W: HasRawWindowHandle>(window: &W, size: (u32, u32), device: &wgpu::Device) -> Self {
        let window_surface = wgpu::Surface::create(window);
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&window_surface, &swap_chain_desc);
        Self {
            window_surface,
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

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::TimeOut> {
        self.swap_chain.get_next_texture().map(SwapChainTargetFrame)
    }
}

#[derive(Debug)]
pub struct TextureTarget {
    size: wgpu::Extent3d,
    texture: wgpu::Texture,
    format: wgpu::TextureFormat,
}

#[derive(Debug)]
pub struct TextureTargetFrame(wgpu::TextureView);

impl RenderTargetFrame for TextureTargetFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.0
    }
}

impl TextureTarget {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };
        let label = create_debug_label!("Render target texture");
        let format = wgpu::TextureFormat::Bgra8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
        });
        Self {
            size,
            texture,
            format,
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
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
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

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::TimeOut> {
        Ok(TextureTargetFrame(self.texture.create_default_view()))
    }
}
