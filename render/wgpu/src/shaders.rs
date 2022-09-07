#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    pub bitmap_shader: wgpu::ShaderModule,
    pub gradient_shader: wgpu::ShaderModule,
    pub copy_srgb_shader: wgpu::ShaderModule,
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
        let gradient_shader = create_shader(device, "gradient", gradient_shader);
        let copy_srgb_shader = create_shader(
            device,
            "copy sRGB",
            include_str!("../shaders/copy_srgb.wgsl"),
        );

        Self {
            color_shader,
            bitmap_shader,
            gradient_shader,
            copy_srgb_shader,
        }
    }
}

/// Builds a `wgpu::ShaderModule` the given WGSL source in `src`.
///
/// The source is prepended with common code in `common.wgsl`, simulating a `#include` preprocessor.
/// We could possibly does this as an offline build step instead.
fn create_shader(
    device: &wgpu::Device,
    name: &'static str,
    src: &'static str,
) -> wgpu::ShaderModule {
    const COMMON_SRC: &str = include_str!("../shaders/common.wgsl");
    let src = [COMMON_SRC, src].concat();
    let label = create_debug_label!("Shader {}", name,);
    let desc = wgpu::ShaderModuleDescriptor {
        label: label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    };
    device.create_shader_module(desc)
}
