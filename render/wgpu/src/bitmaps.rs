use crate::layouts::BindLayouts;

#[derive(Debug)]
pub struct BitmapSamplers {
    repeat_linear: wgpu::BindGroup,
    repeat_nearest: wgpu::BindGroup,
    clamp_linear: wgpu::BindGroup,
    clamp_nearest: wgpu::BindGroup,
}

fn create_sampler(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    address_mode: wgpu::AddressMode,
    filter: wgpu::FilterMode,
    sampler_label: Option<String>,
    group_label: Option<String>,
) -> wgpu::BindGroup {
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
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: group_label.as_deref(),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&sampler),
        }],
    })
}

impl BitmapSamplers {
    pub fn new(device: &wgpu::Device, layouts: &BindLayouts) -> Self {
        let layout = &layouts.bitmap_sampler;
        let repeat_linear = create_sampler(
            device,
            &layout,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Linear,
            create_debug_label!("Repeat & Linear sampler"),
            create_debug_label!("Repeat & Linear bind group"),
        );
        let repeat_nearest = create_sampler(
            device,
            &layout,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Nearest,
            create_debug_label!("Repeat & Nearest sampler"),
            create_debug_label!("Repeat & Nearest bind group"),
        );
        let clamp_linear = create_sampler(
            device,
            &layout,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            create_debug_label!("Clamp & Linear sampler"),
            create_debug_label!("Clamp & Linear bind group"),
        );
        let clamp_nearest = create_sampler(
            device,
            &layout,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Nearest,
            create_debug_label!("Clamp & Nearest sampler"),
            create_debug_label!("Clamp & Nearest bind group"),
        );

        Self {
            repeat_linear,
            repeat_nearest,
            clamp_linear,
            clamp_nearest,
        }
    }

    pub fn get_bind_group(&self, is_repeating: bool, is_smoothed: bool) -> &wgpu::BindGroup {
        match (is_repeating, is_smoothed) {
            (true, true) => &self.repeat_linear,
            (true, false) => &self.repeat_nearest,
            (false, true) => &self.clamp_linear,
            (false, false) => &self.clamp_nearest,
        }
    }
}
