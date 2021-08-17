#[derive(Copy, Clone, PartialEq, Debug, clap::ArgEnum)]
pub enum GraphicsBackend {
    Default,
    Vulkan,
    Metal,
    Dx12,
    Dx11,
}

impl From<GraphicsBackend> for wgpu::BackendBit {
    fn from(backend: GraphicsBackend) -> Self {
        match backend {
            GraphicsBackend::Default => wgpu::BackendBit::PRIMARY | wgpu::BackendBit::DX11,
            GraphicsBackend::Vulkan => wgpu::BackendBit::VULKAN,
            GraphicsBackend::Metal => wgpu::BackendBit::METAL,
            GraphicsBackend::Dx12 => wgpu::BackendBit::DX12,
            GraphicsBackend::Dx11 => wgpu::BackendBit::DX11,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, clap::ArgEnum)]
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
