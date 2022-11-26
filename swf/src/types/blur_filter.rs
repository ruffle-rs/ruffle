use crate::Fixed16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlurFilter {
    pub blur_x: Fixed16,
    pub blur_y: Fixed16,
    pub num_passes: u8,
}
