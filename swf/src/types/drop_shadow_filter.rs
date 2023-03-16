use crate::{Color, Fixed16, Fixed8};
use bitflags::bitflags;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropShadowFilter {
    pub color: Color,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub angle: Fixed16,
    pub distance: Fixed16,
    pub strength: Fixed8,
    pub flags: DropShadowFilterFlags,
}

impl DropShadowFilter {
    #[inline]
    pub fn is_inner(&self) -> bool {
        self.flags.contains(DropShadowFilterFlags::INNER_SHADOW)
    }

    #[inline]
    pub fn is_knockout(&self) -> bool {
        self.flags.contains(DropShadowFilterFlags::KNOCKOUT)
    }

    #[inline]
    pub fn num_passes(&self) -> u8 {
        (self.flags & DropShadowFilterFlags::PASSES).bits()
    }

    #[inline]
    pub fn hide_object(&self) -> bool {
        !self.flags.contains(DropShadowFilterFlags::COMPOSITE_SOURCE)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DropShadowFilterFlags: u8 {
        const INNER_SHADOW     = 1 << 7;
        const KNOCKOUT         = 1 << 6;
        const COMPOSITE_SOURCE = 1 << 5;
        const PASSES           = 0b11111;
    }
}

impl DropShadowFilterFlags {
    #[inline]
    pub fn from_passes(num_passes: u8) -> Self {
        let flags = Self::from_bits_retain(num_passes);
        debug_assert_eq!(flags & Self::PASSES, flags);
        flags
    }
}
