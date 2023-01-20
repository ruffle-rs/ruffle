use crate::Fixed16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorMatrixFilter {
    pub matrix: [Fixed16; 20],
}

impl Default for ColorMatrixFilter {
    fn default() -> Self {
        Self {
            matrix: [
                // r
                Fixed16::from_f32(1.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                // g
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(1.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                // b
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(1.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                // a
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(0.0),
                Fixed16::from_f32(1.0),
                Fixed16::from_f32(0.0),
            ],
        }
    }
}
