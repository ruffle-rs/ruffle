#[derive(Debug)]
pub struct BitmapSamplers {
    repeat_linear: wgpu::Sampler,
    repeat_nearest: wgpu::Sampler,
    clamp_linear: wgpu::Sampler,
    clamp_nearest: wgpu::Sampler,
}

fn create_sampler(
    device: &wgpu::Device,
    address_mode: wgpu::AddressMode,
    filter: wgpu::FilterMode,
    sampler_label: Option<String>,
) -> wgpu::Sampler {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: sampler_label.as_deref(),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: None,
        anisotropy_clamp: None,
        border_color: None,
    });
    sampler
}

impl BitmapSamplers {
    pub fn new(device: &wgpu::Device) -> Self {
        let repeat_linear = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Linear,
            create_debug_label!("Repeat & Linear sampler"),
        );
        let repeat_nearest = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Nearest,
            create_debug_label!("Repeat & Nearest sampler"),
        );
        let clamp_linear = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            create_debug_label!("Clamp & Linear sampler"),
        );
        let clamp_nearest = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Nearest,
            create_debug_label!("Clamp & Nearest sampler"),
        );

        Self {
            repeat_linear,
            repeat_nearest,
            clamp_linear,
            clamp_nearest,
        }
    }

    pub fn get_sampler(&self, is_repeating: bool, is_smoothed: bool) -> &wgpu::Sampler {
        match (is_repeating, is_smoothed) {
            (true, true) => &self.repeat_linear,
            (true, false) => &self.repeat_nearest,
            (false, true) => &self.clamp_linear,
            (false, false) => &self.clamp_nearest,
        }
    }
}
