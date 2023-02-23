use swf::{BevelFilterFlags, Color, Fixed16};

#[derive(Debug, Clone)]
pub enum Filter {
    BevelFilter(BevelFilter),
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

#[derive(Debug, Clone, Copy)]
pub enum BevelFilterType {
    Inner,
    Outer,
    Full,
}

#[derive(Debug, Clone)]
pub struct BevelFilter {
    pub shadow_color: Color,
    pub highlight_color: Color,
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: u8,
    pub bevel_type: BevelFilterType,
    pub knockout: bool,
    pub quality: u8,
}

impl From<swf::BevelFilter> for BevelFilter {
    fn from(value: swf::BevelFilter) -> Self {
        let quality = value.num_passes();
        Self {
            shadow_color: value.shadow_color,
            highlight_color: value.highlight_color,
            blur_x: value.blur_x.to_f32(),
            blur_y: value.blur_y.to_f32(),
            angle: value.angle.to_f32(),
            distance: value.distance.to_f32(),
            strength: (value.strength.to_f32() * 255.0) as u8,
            bevel_type: if value.flags.contains(BevelFilterFlags::ON_TOP) {
                BevelFilterType::Full
            } else if value.flags.contains(BevelFilterFlags::INNER_SHADOW) {
                BevelFilterType::Inner
            } else {
                BevelFilterType::Outer
            },
            knockout: value.flags.contains(BevelFilterFlags::KNOCKOUT),
            quality,
        }
    }
}

impl Default for BevelFilter {
    fn default() -> Self {
        Self {
            shadow_color: Color::BLACK,
            highlight_color: Color::WHITE,
            blur_x: 4.0,
            blur_y: 4.0,
            angle: 45.0,
            distance: 4.0,
            strength: 1,
            bevel_type: BevelFilterType::Inner,
            knockout: false,
            quality: 1,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
