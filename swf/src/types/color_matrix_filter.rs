use crate::Fixed16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorMatrixFilter {
    pub matrix: [Fixed16; 20],
}
