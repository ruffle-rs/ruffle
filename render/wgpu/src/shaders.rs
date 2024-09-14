use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};
use ruffle_render::shader_source::SHADER_FILTER_COMMON;

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    /// This has a pipeline-overridable `bool` constant, `late_saturate`,
    /// with a default of `false`. It switches to performing saturation
    /// after re-multiplying the alpha, rather than before. This is used
    /// for the Stage3D `bitmap_opaque` pipeline, which needs to be able to
    /// avoid changing initially-in-range rgb values (regadless of whether
    /// dividing by the alpha value would produce an out-of-range value).
    pub bitmap_shader: wgpu::ShaderModule,
    pub gradient_shader: wgpu::ShaderModule,
    pub copy_srgb_shader: wgpu::ShaderModule,
    pub copy_shader: wgpu::ShaderModule,
    pub blend_shaders: EnumMap<ComplexBlend, wgpu::ShaderModule>,
    pub color_matrix_filter: wgpu::ShaderModule,
    pub blur_filter: wgpu::ShaderModule,
    pub glow_filter: wgpu::ShaderModule,
    pub bevel_filter: wgpu::ShaderModule,
    pub displacement_map_filter: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let color_shader = make_shader(device, "color.wgsl", include_str!("../shaders/color.wgsl"));
        let bitmap_shader = make_shader(
            device,
            "bitmap.wgsl",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let copy_srgb_shader = make_shader(
            device,
            "copy_srgb.wgsl",
            include_str!("../shaders/copy_srgb.wgsl"),
        );
        let copy_shader = make_shader(device, "copy.wgsl", include_str!("../shaders/copy.wgsl"));
        let color_matrix_filter = make_filter_shader(
            device,
            "filter/color_matrix.wgsl",
            include_str!("../shaders/filter/color_matrix.wgsl"),
        );
        let blur_filter = make_filter_shader(
            device,
            "filter/blur.wgsl",
            include_str!("../shaders/filter/blur.wgsl"),
        );
        let glow_filter = make_filter_shader(
            device,
            "filter/glow.wgsl",
            include_str!("../shaders/filter/glow.wgsl"),
        );
        let bevel_filter = make_filter_shader(
            device,
            "filter/bevel.wgsl",
            include_str!("../shaders/filter/bevel.wgsl"),
        );
        let displacement_map_filter = make_filter_shader(
            device,
            "filter/displacement_map.wgsl",
            include_str!("../shaders/filter/displacement_map.wgsl"),
        );
        let gradient_shader = make_shader(
            device,
            "gradient.wgsl",
            include_str!("../shaders/gradient.wgsl"),
        );

        let blend_shaders = enum_map! {
            ComplexBlend::Multiply => make_shader(device, "blend/multiply.wgsl", include_str!("../shaders/blend/multiply.wgsl")),
            ComplexBlend::Lighten => make_shader(device, "blend/lighten.wgsl", include_str!("../shaders/blend/lighten.wgsl")),
            ComplexBlend::Darken => make_shader(device, "blend/darken.wgsl", include_str!("../shaders/blend/darken.wgsl")),
            ComplexBlend::Difference => make_shader(device, "blend/difference.wgsl", include_str!("../shaders/blend/difference.wgsl")),
            ComplexBlend::Invert => make_shader(device, "blend/invert.wgsl", include_str!("../shaders/blend/invert.wgsl")),
            ComplexBlend::Alpha => make_shader(device, "blend/alpha.wgsl", include_str!("../shaders/blend/alpha.wgsl")),
            ComplexBlend::Erase => make_shader(device, "blend/erase.wgsl", include_str!("../shaders/blend/erase.wgsl")),
            ComplexBlend::Overlay => make_shader(device, "blend/overlay.wgsl", include_str!("../shaders/blend/overlay.wgsl")),
            ComplexBlend::HardLight => make_shader(device, "blend/hardlight.wgsl", include_str!("../shaders/blend/hardlight.wgsl")),
        };

        Self {
            color_shader,
            bitmap_shader,
            gradient_shader,
            copy_srgb_shader,
            copy_shader,
            blend_shaders,
            color_matrix_filter,
            blur_filter,
            glow_filter,
            bevel_filter,
            displacement_map_filter,
        }
    }
}

fn make_shader(device: &wgpu::Device, name: &str, source: &str) -> wgpu::ShaderModule {
    let common = include_str!("../shaders/common.wgsl");
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: create_debug_label!("Shader {}", name).as_deref(),
        source: wgpu::ShaderSource::Wgsl(format!("{}\n{}", common, source).into()),
    })
}
fn make_filter_shader(device: &wgpu::Device, name: &str, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: create_debug_label!("Shader {}", name).as_deref(),
        source: wgpu::ShaderSource::Wgsl(format!("{}\n{}", SHADER_FILTER_COMMON, source).into()),
    })
}
