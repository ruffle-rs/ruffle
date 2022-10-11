use crate::{Color, Fixed16};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvolutionFilter {
    pub num_matrix_rows: u8,
    pub num_matrix_cols: u8,
    pub matrix: Vec<Fixed16>,
    pub divisor: Fixed16,
    pub bias: Fixed16,
    pub default_color: Color,
    pub is_clamped: bool,
    pub is_preserve_alpha: bool,
}
