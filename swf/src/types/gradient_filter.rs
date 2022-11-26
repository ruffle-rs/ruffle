use crate::{Fixed16, Fixed8, GradientRecord};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GradientFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub angle: Fixed16,
    pub distance: Fixed16,
    pub strength: Fixed8,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub is_on_top: bool,
    pub num_passes: u8,
}
