use crate::{Color, Fixed16, Fixed8};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropShadowFilter {
    pub color: Color,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub angle: Fixed16,
    pub distance: Fixed16,
    pub strength: Fixed8,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub num_passes: u8,
}
