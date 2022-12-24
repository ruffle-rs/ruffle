use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    pub bitmap_shader: wgpu::ShaderModule,
    pub gradient_shader: wgpu::ShaderModule,
    pub copy_srgb_shader: wgpu::ShaderModule,
    pub copy_shader: wgpu::ShaderModule,
    pub blend_shaders: EnumMap<ComplexBlend, wgpu::ShaderModule>,
}

impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let color_shader = create_shader(device, "color", include_str!("../shaders/color.wgsl"));
        let bitmap_shader = create_shader(device, "bitmap", include_str!("../shaders/bitmap.wgsl"));
        let gradient_shader = if device.limits().max_storage_buffers_per_shader_stage > 0 {
            include_str!("../shaders/gradient_storage.wgsl")
        } else {
            include_str!("../shaders/gradient_uniform.wgsl")
        };
        let gradient_shader = create_gradient_shader(device, "gradient", gradient_shader);
        let copy_srgb_shader = create_shader(
            device,
            "copy sRGB",
            include_str!("../shaders/copy_srgb.wgsl"),
        );
        let copy_shader = create_shader(device, "copy", include_str!("../shaders/copy.wgsl"));

        let blend_shaders = enum_map! {
            ComplexBlend::Multiply => create_shader(device, "blend - multiply", include_str!("../shaders/blend/multiply.wgsl")),
            ComplexBlend::Screen => create_shader(device, "blend - screen", include_str!("../shaders/blend/screen.wgsl")),
            ComplexBlend::Lighten => create_shader(device, "blend - lighten", include_str!("../shaders/blend/lighten.wgsl")),
            ComplexBlend::Darken => create_shader(device, "blend - darken", include_str!("../shaders/blend/darken.wgsl")),
            ComplexBlend::Difference => create_shader(device, "blend - difference", include_str!("../shaders/blend/difference.wgsl")),
            ComplexBlend::Invert => create_shader(device, "blend - invert", include_str!("../shaders/blend/invert.wgsl")),
            ComplexBlend::Alpha => create_shader(device, "blend - alpha", include_str!("../shaders/blend/alpha.wgsl")),
            ComplexBlend::Erase => create_shader(device, "blend - erase", include_str!("../shaders/blend/erase.wgsl")),
            ComplexBlend::Overlay => create_shader(device, "blend - overlay", include_str!("../shaders/blend/overlay.wgsl")),
            ComplexBlend::HardLight => create_shader(device, "blend - hardlight", include_str!("../shaders/blend/hardlight.wgsl")),
        };

        Self {
            color_shader,
            bitmap_shader,
            gradient_shader,
            copy_srgb_shader,
            copy_shader,
            blend_shaders,
        }
    }
}

/// Builds a `wgpu::ShaderModule` the given WGSL source in `src`.
///
/// The source is prepended with common code in `common.wgsl`, simulating a `#include` preprocessor.
/// We could possibly does this as an offline build step instead.
fn create_shader(device: &wgpu::Device, name: &str, src: &str) -> wgpu::ShaderModule {
    const COMMON_SRC: &str = include_str!("../shaders/common.wgsl");
    let src = [COMMON_SRC, src].concat();
    let label = create_debug_label!("Shader {}", name);
    let desc = wgpu::ShaderModuleDescriptor {
        label: label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    };
    device.create_shader_module(desc)
}

fn create_gradient_shader(device: &wgpu::Device, name: &str, src: &str) -> wgpu::ShaderModule {
    const COMMON_SRC: &str = include_str!("../shaders/gradient/common.wgsl");
    let src = [src, COMMON_SRC].concat();

    create_shader(device, name, &src)
}
