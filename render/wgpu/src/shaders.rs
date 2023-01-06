use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};
use ruffle_render::tessellator::GradientType;
use swf::GradientSpread;

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    pub bitmap_shader: wgpu::ShaderModule,
    pub gradient_shaders: EnumMap<GradientType, EnumMap<GradientSpread, wgpu::ShaderModule>>,
    pub copy_srgb_shader: wgpu::ShaderModule,
    pub copy_shader: wgpu::ShaderModule,
    pub blend_shaders: EnumMap<ComplexBlend, wgpu::ShaderModule>,
}

impl Shaders {
    pub fn new(device: &wgpu::Device, push_constants: bool) -> Self {
        let color_shader = create_shader(
            device,
            push_constants,
            "color",
            include_str!("../shaders/color.wgsl"),
        );
        let bitmap_shader = create_shader(
            device,
            push_constants,
            "bitmap",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let copy_srgb_shader = create_shader(
            device,
            push_constants,
            "copy sRGB",
            include_str!("../shaders/copy_srgb.wgsl"),
        );
        let copy_shader = create_shader(
            device,
            push_constants,
            "copy",
            include_str!("../shaders/copy.wgsl"),
        );

        let blend_shaders = enum_map! {
            ComplexBlend::Multiply => create_shader(device, push_constants, "blend - multiply", include_str!("../shaders/blend/multiply.wgsl")),
            ComplexBlend::Screen => create_shader(device, push_constants, "blend - screen", include_str!("../shaders/blend/screen.wgsl")),
            ComplexBlend::Lighten => create_shader(device, push_constants, "blend - lighten", include_str!("../shaders/blend/lighten.wgsl")),
            ComplexBlend::Darken => create_shader(device, push_constants, "blend - darken", include_str!("../shaders/blend/darken.wgsl")),
            ComplexBlend::Difference => create_shader(device, push_constants, "blend - difference", include_str!("../shaders/blend/difference.wgsl")),
            ComplexBlend::Invert => create_shader(device, push_constants, "blend - invert", include_str!("../shaders/blend/invert.wgsl")),
            ComplexBlend::Alpha => create_shader(device, push_constants, "blend - alpha", include_str!("../shaders/blend/alpha.wgsl")),
            ComplexBlend::Erase => create_shader(device, push_constants, "blend - erase", include_str!("../shaders/blend/erase.wgsl")),
            ComplexBlend::Overlay => create_shader(device, push_constants, "blend - overlay", include_str!("../shaders/blend/overlay.wgsl")),
            ComplexBlend::HardLight => create_shader(device, push_constants, "blend - hardlight", include_str!("../shaders/blend/hardlight.wgsl")),
        };

        let gradient_shader = if device.limits().max_storage_buffers_per_shader_stage > 0 {
            include_str!("../shaders/gradient_storage.wgsl")
        } else {
            include_str!("../shaders/gradient_uniform.wgsl")
        };
        let type_focal = include_str!("../shaders/gradient/mode/focal.wgsl");
        let type_linear = include_str!("../shaders/gradient/mode/linear.wgsl");
        let type_radial = include_str!("../shaders/gradient/mode/radial.wgsl");

        let gradient_shaders = enum_map! {
            GradientType::Focal => create_gradient_shaders(device, push_constants, "focal", type_focal, gradient_shader),
            GradientType::Linear => create_gradient_shaders(device, push_constants, "linear", type_linear, gradient_shader),
            GradientType::Radial => create_gradient_shaders(device, push_constants, "radial", type_radial, gradient_shader),
        };

        Self {
            color_shader,
            bitmap_shader,
            gradient_shaders,
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
fn create_shader(
    device: &wgpu::Device,
    push_constants: bool,
    name: &str,
    src: &str,
) -> wgpu::ShaderModule {
    const COMMON_SRC: &str = include_str!("../shaders/common.wgsl");
    const UNIFORMS_PC_SRC: &str = include_str!("../shaders/common_push_constants.wgsl");
    const UNIFORMS_NO_PC_SRC: &str = include_str!("../shaders/common_no_push_constants.wgsl");
    let uniforms = if push_constants {
        UNIFORMS_PC_SRC
    } else {
        UNIFORMS_NO_PC_SRC
    };
    let src = [COMMON_SRC, uniforms, src].concat();
    let label = create_debug_label!("Shader {}", name);
    let desc = wgpu::ShaderModuleDescriptor {
        label: label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    };
    device.create_shader_module(desc)
}

fn create_gradient_shaders(
    device: &wgpu::Device,
    push_constants: bool,
    name: &str,
    mode: &str,
    special: &str,
) -> EnumMap<GradientSpread, wgpu::ShaderModule> {
    const COMMON_SRC: &str = include_str!("../shaders/gradient/common.wgsl");
    const SPREAD_REFLECT: &str = include_str!("../shaders/gradient/repeat/mirror.wgsl");
    const SPREAD_REPEAT: &str = include_str!("../shaders/gradient/repeat/repeat.wgsl");
    const SPREAD_PAD: &str = include_str!("../shaders/gradient/repeat/clamp.wgsl");

    enum_map! {
        GradientSpread::Reflect => create_shader(device, push_constants, &format!("gradient - {name} reflect"), &[mode, SPREAD_REFLECT, special, COMMON_SRC].concat()),
        GradientSpread::Repeat => create_shader(device,push_constants, &format!("gradient - {name} repeat"), &[mode, SPREAD_REPEAT, special, COMMON_SRC].concat()),
        GradientSpread::Pad => create_shader(device, push_constants,&format!("gradient - {name} pad"), &[mode, SPREAD_PAD, special, COMMON_SRC].concat()),
    }
}
