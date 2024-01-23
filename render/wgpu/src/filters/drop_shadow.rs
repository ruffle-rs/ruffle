use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::blur::BlurFilter;
use crate::filters::glow::GlowFilter;
use crate::filters::FilterSource;
use crate::surface::target::CommandTarget;
use swf::DropShadowFilter as DropShadowFilterArgs;
use wgpu::util::StagingBelt;

/// Drop shadow is just Glow with an offset.
/// None of this strictly needs to be a struct,
/// but it helps for code organisation + if we want to specialise the implementation in the future
pub struct DropShadowFilter;

impl DropShadowFilter {
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut StagingBelt,
        source: &FilterSource,
        filter: &DropShadowFilterArgs,
        blur_filter: &BlurFilter,
        glow_filter: &GlowFilter,
    ) -> CommandTarget {
        let distance = filter.distance.to_f32();
        let angle = filter.angle.to_f32();
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        glow_filter.apply(
            descriptors,
            texture_pool,
            draw_encoder,
            staging_belt,
            source,
            &filter.inner_glow_filter(),
            blur_filter,
            (-x, -y),
        )
    }
}
