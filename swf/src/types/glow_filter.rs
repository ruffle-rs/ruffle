use crate::{Color, Fixed16, Fixed8};
use bitflags::bitflags;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlowFilter {
    pub color: Color,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub strength: Fixed8,
    pub flags: GlowFilterFlags,
}

impl GlowFilter {
    #[inline]
    pub fn is_inner(&self) -> bool {
        self.flags.contains(GlowFilterFlags::INNER_GLOW)
    }

    #[inline]
    pub fn is_knockout(&self) -> bool {
        self.flags.contains(GlowFilterFlags::KNOCKOUT)
    }

    #[inline]
    pub fn num_passes(&self) -> u8 {
        (self.flags & GlowFilterFlags::PASSES).bits()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct GlowFilterFlags: u8 {
        const INNER_GLOW       = 1 << 7;
        const KNOCKOUT         = 1 << 6;
        const COMPOSITE_SOURCE = 1 << 5;
        const PASSES           = 0b11111;
    }
}

impl GlowFilterFlags {
    #[inline]
    pub fn from_passes(num_passes: u8) -> Self {
        let flags = Self::from_bits_retain(num_passes);
        debug_assert_eq!(flags & Self::PASSES, flags);
        flags
    }
}
