use crate::Color;
use bitflags::bitflags;

#[derive(Clone, Debug, PartialEq)]
pub struct ConvolutionFilter {
    pub num_matrix_rows: u8,
    pub num_matrix_cols: u8,
    pub matrix: Vec<f32>,
    pub divisor: f32,
    pub bias: f32,
    pub default_color: Color,
    pub flags: ConvolutionFilterFlags,
}

impl ConvolutionFilter {
    #[inline]
    pub fn is_clamped(&self) -> bool {
        self.flags.contains(ConvolutionFilterFlags::CLAMP)
    }

    #[inline]
    pub fn is_preserve_alpha(&self) -> bool {
        self.flags.contains(ConvolutionFilterFlags::PRESERVE_ALPHA)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ConvolutionFilterFlags: u8 {
        const CLAMP          = 1 << 1;
        const PRESERVE_ALPHA = 1 << 0;
    }
}
