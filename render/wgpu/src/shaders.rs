use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor, ShaderDefValue,
};
use ruffle_render::tessellator::GradientType;
use std::borrow::Cow;
use std::collections::HashMap;
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
    pub fn new(device: &wgpu::Device) -> Self {
        let mut composer = composer().expect("Couldn't create shader composer");
        let mut shader_defs = HashMap::new();
        shader_defs.insert(
            "use_push_constants".to_owned(),
            ShaderDefValue::Bool(device.limits().max_push_constant_size > 0),
        );
        shader_defs.insert(
            "use_storage_buffers".to_owned(),
            ShaderDefValue::Bool(device.limits().max_storage_buffers_per_shader_stage > 0),
        );
        let color_shader = make_shader(
            &device,
            &mut composer,
            &shader_defs,
            "color.wgsl",
            include_str!("../shaders/color.wgsl"),
        );
        let bitmap_shader = make_shader(
            &device,
            &mut composer,
            &shader_defs,
            "bitmap.wgsl",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let copy_srgb_shader = make_shader(
            &device,
            &mut composer,
            &shader_defs,
            "copy_srgb.wgsl",
            include_str!("../shaders/copy_srgb.wgsl"),
        );
        let copy_shader = make_shader(
            &device,
            &mut composer,
            &shader_defs,
            "copy.wgsl",
            include_str!("../shaders/copy.wgsl"),
        );

        let blend_shaders = enum_map! {
            ComplexBlend::Multiply => make_shader(device, &mut composer, &shader_defs, "blend/multiply.wgsl", include_str!("../shaders/blend/multiply.wgsl")),
            ComplexBlend::Screen => make_shader(device, &mut composer, &shader_defs, "blend/screen.wgsl", include_str!("../shaders/blend/screen.wgsl")),
            ComplexBlend::Lighten => make_shader(device, &mut composer, &shader_defs, "blend/lighten.wgsl", include_str!("../shaders/blend/lighten.wgsl")),
            ComplexBlend::Darken => make_shader(device, &mut composer, &shader_defs, "blend/darken.wgsl", include_str!("../shaders/blend/darken.wgsl")),
            ComplexBlend::Difference => make_shader(device, &mut composer, &shader_defs, "blend/difference.wgsl", include_str!("../shaders/blend/difference.wgsl")),
            ComplexBlend::Invert => make_shader(device, &mut composer, &shader_defs, "blend/invert.wgsl", include_str!("../shaders/blend/invert.wgsl")),
            ComplexBlend::Alpha => make_shader(device, &mut composer, &shader_defs, "blend/alpha.wgsl", include_str!("../shaders/blend/alpha.wgsl")),
            ComplexBlend::Erase => make_shader(device, &mut composer, &shader_defs, "blend/erase.wgsl", include_str!("../shaders/blend/erase.wgsl")),
            ComplexBlend::Overlay => make_shader(device, &mut composer, &shader_defs, "blend/overlay.wgsl", include_str!("../shaders/blend/overlay.wgsl")),
            ComplexBlend::HardLight => make_shader(device, &mut composer, &shader_defs, "blend/hardlight.wgsl", include_str!("../shaders/blend/hardlight.wgsl")),
        };

        let gradient_shaders = enum_map! {
            GradientType::Focal => create_gradient_shaders(device, &mut composer, &shader_defs, "focal", include_str!("../shaders/gradient/mode/focal.wgsl")),
            GradientType::Linear => create_gradient_shaders(device, &mut composer, &shader_defs, "linear", include_str!("../shaders/gradient/mode/linear.wgsl")),
            GradientType::Radial => create_gradient_shaders(device, &mut composer, &shader_defs, "radial", include_str!("../shaders/gradient/mode/radial.wgsl")),
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

fn composer() -> Result<Composer, ComposerError> {
    let mut composer = Composer::default();
    // [NA] Hack to get all capabilities since nobody exposes this type easily
    let capabilities = composer.capabilities;
    composer = composer.with_capabilities(!capabilities);
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("../shaders/common.wgsl"),
        file_path: "common.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("../shaders/gradient/common.wgsl"),
        file_path: "gradient/common.wgsl",
        ..Default::default()
    })?;
    Ok(composer)
}

fn make_shader(
    device: &wgpu::Device,
    composer: &mut Composer,
    shader_defs: &HashMap<String, ShaderDefValue>,
    name: &str,
    source: &'static str,
) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: create_debug_label!("Shader {}", name).as_deref(),
        source: wgpu::ShaderSource::Naga(Cow::Owned(
            composer
                .make_naga_module(NagaModuleDescriptor {
                    source,
                    file_path: name,
                    shader_defs: shader_defs.clone(),
                    ..Default::default()
                })
                .unwrap_or_else(|_| panic!("{name} failed to compile")),
        )),
    })
}

fn create_gradient_shaders(
    device: &wgpu::Device,
    composer: &mut Composer,
    shader_defs: &HashMap<String, ShaderDefValue>,
    name: &'static str,
    source: &'static str,
) -> EnumMap<GradientSpread, wgpu::ShaderModule> {
    enum_map! {
        GradientSpread::Reflect => {
            let mut temporary_defs = shader_defs.clone();
            temporary_defs.insert("gradient_repeat_mode".to_owned(), ShaderDefValue::Int(1));
            make_shader(device, composer, &temporary_defs, &format!("gradient/{name}.wgsl with reflect"), source)
        },
        GradientSpread::Repeat => {
            let mut temporary_defs = shader_defs.clone();
            temporary_defs.insert("gradient_repeat_mode".to_owned(), ShaderDefValue::Int(2));
            make_shader(device, composer, &temporary_defs, &format!("gradient/{name}.wgsl with repeat"), source)
        },
        GradientSpread::Pad => {
            let mut temporary_defs = shader_defs.clone();
            temporary_defs.insert("gradient_repeat_mode".to_owned(), ShaderDefValue::Int(3));
            make_shader(device, composer, &temporary_defs,&format!("gradient/{name}.wgsl with pad"), source)
        },
    }
}
