use crate::{Color, Fixed16, Fixed8};
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
