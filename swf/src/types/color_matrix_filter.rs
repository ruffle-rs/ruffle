#[derive(Clone, Debug, PartialEq)]
pub struct ColorMatrixFilter {
    pub matrix: [f32; 20],
}

impl Default for ColorMatrixFilter {
    fn default() -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, // r
                0.0, 1.0, 0.0, 0.0, 0.0, // g
                0.0, 0.0, 1.0, 0.0, 0.0, // b
                0.0, 0.0, 0.0, 1.0, 0.0, //a
            ],
        }
    }
}

impl ColorMatrixFilter {
    pub fn impotent(&self) -> bool {
        self == &Default::default()
    }
}
