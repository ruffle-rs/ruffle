use std::str::FromStr;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum GraphicsBackend {
    #[default]
    Default,
    Vulkan,
    Metal,
    Dx12,
    Gl,
}

impl GraphicsBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            GraphicsBackend::Default => "default",
            GraphicsBackend::Vulkan => "vulkan",
            GraphicsBackend::Metal => "metal",
            GraphicsBackend::Dx12 => "dx12",
            GraphicsBackend::Gl => "gl",
        }
    }
}

impl FromStr for GraphicsBackend {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(GraphicsBackend::Default),
            "vulkan" => Ok(GraphicsBackend::Vulkan),
            "metal" => Ok(GraphicsBackend::Metal),
            "dx12" => Ok(GraphicsBackend::Dx12),
            "gl" => Ok(GraphicsBackend::Gl),
            _ => Err(()),
        }
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
pub enum PowerPreference {
    Low,
    #[default]
    High,
}

impl PowerPreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            PowerPreference::High => "high",
            PowerPreference::Low => "low",
        }
    }
}

impl FromStr for PowerPreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(PowerPreference::High),
            "low" => Ok(PowerPreference::Low),
            _ => Err(()),
        }
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
