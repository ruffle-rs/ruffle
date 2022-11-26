#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphicsBackend {
    Default,
    Vulkan,
    Metal,
    Dx12,
    Gl,
}

impl From<GraphicsBackend> for wgpu::Backends {
    fn from(backend: GraphicsBackend) -> Self {
        match backend {
            GraphicsBackend::Default => wgpu::Backends::PRIMARY | wgpu::Backends::DX11,
            GraphicsBackend::Vulkan => wgpu::Backends::VULKAN,
            GraphicsBackend::Metal => wgpu::Backends::METAL,
            GraphicsBackend::Dx12 => wgpu::Backends::DX12,
            GraphicsBackend::Gl => wgpu::Backends::GL,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerPreference {
    Low = 1,
    High = 2,
}

impl From<PowerPreference> for wgpu::PowerPreference {
    fn from(preference: PowerPreference) -> Self {
        match preference {
            PowerPreference::Low => wgpu::PowerPreference::LowPower,
            PowerPreference::High => wgpu::PowerPreference::HighPerformance,
        }
    }
}
