use crate::{BlurFilter, BlurFilterFlags, Color, Fixed16, Fixed8, Rectangle, Twips};
use bitflags::bitflags;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BevelFilter {
    pub shadow_color: Color,
    pub highlight_color: Color,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub angle: Fixed16,
    pub distance: Fixed16,
    pub strength: Fixed8,
    pub flags: BevelFilterFlags,
}

impl BevelFilter {
    #[inline]
    pub fn is_inner(&self) -> bool {
        self.flags.contains(BevelFilterFlags::INNER_SHADOW)
    }

    #[inline]
    pub fn is_knockout(&self) -> bool {
        self.flags.contains(BevelFilterFlags::KNOCKOUT)
    }

    #[inline]
    pub fn is_on_top(&self) -> bool {
        self.flags.contains(BevelFilterFlags::ON_TOP)
    }

    #[inline]
    pub fn num_passes(&self) -> u8 {
        (self.flags & BevelFilterFlags::PASSES).bits()
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.blur_x = BlurFilter::scale_blur(self.blur_x, x);
        self.blur_y = BlurFilter::scale_blur(self.blur_y, y);
        self.distance *= Fixed16::from_f32(y);
    }

    pub fn inner_blur_filter(&self) -> BlurFilter {
        BlurFilter {
            blur_x: self.blur_x,
            blur_y: self.blur_y,
            flags: BlurFilterFlags::from_passes(self.num_passes()),
        }
    }

    pub fn calculate_dest_rect(&self, source_rect: Rectangle<Twips>) -> Rectangle<Twips> {
        let mut result = self.inner_blur_filter().calculate_dest_rect(source_rect);
        let distance = self.distance.to_f64();
        let angle = self.angle.to_f64();
        let x = Twips::from_pixels(angle.cos() * distance);
        let y = Twips::from_pixels(angle.sin() * distance);
        if x < Twips::ZERO {
            result.x_min += x;
            result.x_max -= x;
        } else {
            result.x_max += x;
            result.x_min -= x;
        }
        if y < Twips::ZERO {
            result.y_min += y;
            result.y_max -= y;
        } else {
            result.y_max += y;
            result.y_min -= y;
        }
        result
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct BevelFilterFlags: u8 {
        const INNER_SHADOW     = 1 << 7;
        const KNOCKOUT         = 1 << 6;
        const COMPOSITE_SOURCE = 1 << 5;
        const ON_TOP           = 1 << 4;
        const PASSES           = 0b1111;
    }
}

impl BevelFilterFlags {
    #[inline]
    pub fn from_passes(num_passes: u8) -> Self {
        let flags = Self::from_bits_retain(num_passes);
        debug_assert_eq!(flags & Self::PASSES, flags);
        flags
    }
}
