use clap::Clap;

#[derive(Copy, Clone, Clap, PartialEq, Debug)]
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
            GraphicsBackend::Default => wgpu::BackendBit::PRIMARY,
            GraphicsBackend::Vulkan => wgpu::BackendBit::VULKAN,
            GraphicsBackend::Metal => wgpu::BackendBit::METAL,
            GraphicsBackend::Dx12 => wgpu::BackendBit::DX12,
            GraphicsBackend::Dx11 => wgpu::BackendBit::DX11,
        }
    }
}

#[derive(Copy, Clone, Clap, PartialEq, Debug)]
pub enum PowerPreference {
    Default = 0,
    Low = 1,
    High = 2,
}

impl From<PowerPreference> for wgpu::PowerPreference {
    fn from(preference: PowerPreference) -> Self {
        match preference {
            PowerPreference::Default => wgpu::PowerPreference::default(),
            PowerPreference::Low => wgpu::PowerPreference::LowPower,
            PowerPreference::High => wgpu::PowerPreference::HighPerformance,
        }
    }
}
