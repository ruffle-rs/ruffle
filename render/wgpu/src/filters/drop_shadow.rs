use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::blur::BlurFilter;
use crate::filters::glow::GlowFilter;
use crate::filters::FilterSource;
use crate::surface::target::CommandTarget;
use swf::{DropShadowFilter as DropShadowFilterArgs, Rectangle};

/// Drop shadow is just Glow with an offset.
/// None of this strictly needs to be a struct,
/// but it helps for code organisation + if we want to specialise the implementation in the future
pub struct DropShadowFilter;

impl DropShadowFilter {
    pub fn calculate_dest_rect(
        filter: &DropShadowFilterArgs,
        source_rect: Rectangle<i32>,
        blur_filter: &BlurFilter,
        glow_filter: &GlowFilter,
    ) -> Rectangle<i32> {
        let mut result =
            glow_filter.calculate_dest_rect(&filter.inner_glow_filter(), source_rect, blur_filter);
        let distance = filter.distance.to_f32();
        let angle = filter.angle.to_f32();
        let x = (angle.cos() * distance).ceil() as i32;
        let y = (angle.sin() * distance).ceil() as i32;
        if x < 0 {
            result.x_min += x;
        } else {
            result.x_max += x;
        }
        if y < 0 {
            result.y_min += y;
        } else {
            result.y_max += y;
        }
        result
    }

    pub fn apply(
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
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
            source,
            &filter.inner_glow_filter(),
            blur_filter,
            (-x, -y),
        )
    }
}
