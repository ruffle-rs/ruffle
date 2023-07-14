use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor, ShaderDefValue,
};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    pub bitmap_shader: wgpu::ShaderModule,
    /// Like `bitmap_shader` but performs saturation after we've
    /// re-multiplied the alpha. This is used for the Stage3D
    /// `bitmap_opaque` pipeline, which needs to able to
    /// avoid changing initially-in-range rgb values (regadless
    /// of whether dividing by the alpha value would produce
    /// an out-of-range value).
    pub bitmap_late_saturate_shader: wgpu::ShaderModule,
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
        let mut composer = composer().expect("Couldn't create shader composer");
        let mut shader_defs = HashMap::new();
        shader_defs.insert(
            "use_push_constants".to_owned(),
            ShaderDefValue::Bool(device.limits().max_push_constant_size > 0),
        );
        shader_defs.insert("early_saturate".to_owned(), ShaderDefValue::Bool(true));

        let mut late_saturate_shader_defs = shader_defs.clone();
        late_saturate_shader_defs.insert("early_saturate".to_owned(), ShaderDefValue::Bool(false));

        let color_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "color.wgsl",
            include_str!("../shaders/color.wgsl"),
        );
        let bitmap_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "bitmap.wgsl",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let bitmap_late_saturate_shader = make_shader(
            device,
            &mut composer,
            &late_saturate_shader_defs,
            "bitmap.wgsl",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let copy_srgb_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "copy_srgb.wgsl",
            include_str!("../shaders/copy_srgb.wgsl"),
        );
        let copy_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "copy.wgsl",
            include_str!("../shaders/copy.wgsl"),
        );
        let color_matrix_filter = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "filter/color_matrix.wgsl",
            include_str!("../shaders/filter/color_matrix.wgsl"),
        );
        let blur_filter = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "filter/blur.wgsl",
            include_str!("../shaders/filter/blur.wgsl"),
        );
        let glow_filter = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "filter/glow.wgsl",
            include_str!("../shaders/filter/glow.wgsl"),
        );
        let bevel_filter = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "filter/bevel.wgsl",
            include_str!("../shaders/filter/bevel.wgsl"),
        );
        let displacement_map_filter = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "filter/displacement_map.wgsl",
            include_str!("../shaders/filter/displacement_map.wgsl"),
        );
        let gradient_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "gradient.wgsl",
            include_str!("../shaders/gradient.wgsl"),
        );

        let blend_shaders = enum_map! {
            ComplexBlend::Multiply => make_shader(device, &mut composer, &shader_defs, "blend/multiply.wgsl", include_str!("../shaders/blend/multiply.wgsl")),
            ComplexBlend::Lighten => make_shader(device, &mut composer, &shader_defs, "blend/lighten.wgsl", include_str!("../shaders/blend/lighten.wgsl")),
            ComplexBlend::Darken => make_shader(device, &mut composer, &shader_defs, "blend/darken.wgsl", include_str!("../shaders/blend/darken.wgsl")),
            ComplexBlend::Difference => make_shader(device, &mut composer, &shader_defs, "blend/difference.wgsl", include_str!("../shaders/blend/difference.wgsl")),
            ComplexBlend::Invert => make_shader(device, &mut composer, &shader_defs, "blend/invert.wgsl", include_str!("../shaders/blend/invert.wgsl")),
            ComplexBlend::Alpha => make_shader(device, &mut composer, &shader_defs, "blend/alpha.wgsl", include_str!("../shaders/blend/alpha.wgsl")),
            ComplexBlend::Erase => make_shader(device, &mut composer, &shader_defs, "blend/erase.wgsl", include_str!("../shaders/blend/erase.wgsl")),
            ComplexBlend::Overlay => make_shader(device, &mut composer, &shader_defs, "blend/overlay.wgsl", include_str!("../shaders/blend/overlay.wgsl")),
            ComplexBlend::HardLight => make_shader(device, &mut composer, &shader_defs, "blend/hardlight.wgsl", include_str!("../shaders/blend/hardlight.wgsl")),
        };

        Self {
            color_shader,
            bitmap_shader,
            bitmap_late_saturate_shader,
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
        source: ruffle_render::shader_source::SHADER_FILTER_COMMON,
        file_path: "shader_filter_common.wgsl",
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
                .unwrap_or_else(|e| {
                    panic!(
                        "{name} failed to compile:\n{}\n{:#?}",
                        e.emit_to_string(composer),
                        e
                    )
                }),
        )),
    })
}
