use swf::Fixed16;

#[derive(Debug)]
pub enum Filter {
    BlurFilter(BlurFilter),
    ColorMatrixFilter(ColorMatrixFilter),
}

impl Default for Filter {
    fn default() -> Self {
        // A default colormatrix is a filter that essentially does nothing,
        // making it a useful default in situations that we need a dummy filter
        Filter::ColorMatrixFilter(ColorMatrixFilter::default())
    }
}

#[derive(Debug)]
pub struct BlurFilter {
    pub blur_x: f32,
    pub blur_y: f32,
    pub quality: u8,
}

impl From<swf::BlurFilter> for BlurFilter {
    fn from(value: swf::BlurFilter) -> Self {
        Self {
            blur_x: value.blur_x.to_f32(),
            blur_y: value.blur_y.to_f32(),
            quality: value.num_passes(),
        }
    }
}

impl Default for BlurFilter {
    fn default() -> Self {
        Self {
            blur_x: 4.0,
            blur_y: 4.0,
            quality: 1,
        }
    }
}

#[derive(Debug)]
pub struct ColorMatrixFilter {
    pub matrix: [f32; 20],
}

impl From<swf::ColorMatrixFilter> for ColorMatrixFilter {
    fn from(value: swf::ColorMatrixFilter) -> Self {
        Self {
            matrix: value.matrix.map(Fixed16::to_f32),
        }
    }
}

impl Default for ColorMatrixFilter {
    fn default() -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, // r
                0.0, 1.0, 0.0, 0.0, 0.0, // g
                0.0, 0.0, 1.0, 0.0, 0.0, // b
                0.0, 0.0, 0.0, 1.0, 0.0, // a
            ],
        }
    }
}
