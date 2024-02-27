#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GraphicsBackend {
    #[default]
    Default,
    Vulkan,
    Metal,
    Dx12,
    Gl,
}

impl GraphicsBackend {
    pub fn is_default(&self) -> bool {
        self == &GraphicsBackend::Default
    }
}

impl From<GraphicsBackend> for wgpu::Backends {
    fn from(backend: GraphicsBackend) -> Self {
        match backend {
            GraphicsBackend::Default => wgpu::Backends::PRIMARY,
            GraphicsBackend::Vulkan => wgpu::Backends::VULKAN,
            GraphicsBackend::Metal => wgpu::Backends::METAL,
            GraphicsBackend::Dx12 => wgpu::Backends::DX12,
            GraphicsBackend::Gl => wgpu::Backends::GL,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PowerPreference {
    Low,
    #[default]
    High,
}

impl PowerPreference {
    pub fn is_default(&self) -> bool {
        self == &PowerPreference::High
    }
}

impl From<PowerPreference> for wgpu::PowerPreference {
    fn from(preference: PowerPreference) -> Self {
        match preference {
            PowerPreference::Low => wgpu::PowerPreference::LowPower,
            PowerPreference::High => wgpu::PowerPreference::HighPerformance,
        }
    }
}
