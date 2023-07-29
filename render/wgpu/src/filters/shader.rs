use ruffle_render::{
    filters::ShaderFilter as ShaderFilterArgs,
    pixel_bender::{ImageInputTexture, PixelBenderShaderArgument},
};

use crate::{
    backend::RenderTargetMode,
    buffer_pool::TexturePool,
    descriptors::Descriptors,
    pixel_bender::{run_pixelbender_shader_impl, ShaderMode},
    surface::target::CommandTarget,
};

use super::FilterSource;

/// All of the data is stored in the `ShaderFilterArgs`
#[derive(Default)]
pub struct ShaderFilter;

impl ShaderFilter {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply<'a>(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        source: &FilterSource<'a>,
        mut filter: ShaderFilterArgs<'a>,
    ) -> CommandTarget {
        let sample_count = source.texture.sample_count();
        let format = source.texture.format();

        let target = CommandTarget::new(
            descriptors,
            texture_pool,
            wgpu::Extent3d {
                width: source.size.0,
                height: source.size.1,
                depth_or_array_layers: 1,
            },
            format,
            sample_count,
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            draw_encoder,
        );

        for arg in &mut filter.shader_args {
            if let PixelBenderShaderArgument::ImageInput { texture, .. } = arg {
                *texture = Some(ImageInputTexture::TextureRef(source.texture));
                // Only bind the first input from the source texture
                break;
            }
        }

        run_pixelbender_shader_impl(
            descriptors,
            filter.shader,
            ShaderMode::Filter,
            &filter.shader_args,
            target.color_texture(),
            draw_encoder,
            target.color_attachments(),
            target.sample_count(),
            source,
        )
        .expect("Failed to run pixelbender shader");
        target
    }
}
