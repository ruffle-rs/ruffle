use crate::bitmap::BitmapHandle;
use swf::{BevelFilterFlags, Color, Fixed16, GradientFilterFlags, GradientRecord};

#[derive(Debug, Clone)]
pub enum Filter {
    BevelFilter(BevelFilter),
    BlurFilter(BlurFilter),
    ColorMatrixFilter(ColorMatrixFilter),
    ConvolutionFilter(ConvolutionFilter),
    DisplacementMapFilter(DisplacementMapFilter),
    DropShadowFilter(DropShadowFilter),
    GlowFilter(GlowFilter),
    GradientBevelFilter(GradientBevelFilter),
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

#[derive(Debug, Clone)]
pub struct ConvolutionFilter {
    pub bias: f32,
    pub clamp: bool,
    pub default_color: Color,
    pub divisor: f32,
    pub matrix: Vec<f32>,
    pub matrix_x: u8,
    pub matrix_y: u8,
    pub preserve_alpha: bool,
}

impl From<swf::ConvolutionFilter> for ConvolutionFilter {
    fn from(value: swf::ConvolutionFilter) -> Self {
        let preserve_alpha = value.is_preserve_alpha();
        let clamp = value.is_clamped();
        Self {
            bias: value.bias.to_f32(),
            clamp,
            default_color: value.default_color,
            divisor: value.divisor.to_f32(),
            matrix: value.matrix.iter().map(|v| v.to_f32()).collect(),
            matrix_x: value.num_matrix_cols,
            matrix_y: value.num_matrix_rows,
            preserve_alpha,
        }
    }
}

impl Default for ConvolutionFilter {
    fn default() -> Self {
        Self {
            bias: 0.0,
            clamp: true,
            default_color: Color::from_rgba(0),
            divisor: 1.0,
            matrix: vec![],
            matrix_x: 0,
            matrix_y: 0,
            preserve_alpha: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DisplacementMapFilterComponent {
    Alpha,
    Blue,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy)]
pub enum DisplacementMapFilterMode {
    Clamp,
    Color,
    Ignore,
    Wrap,
}

#[derive(Debug, Clone)]
pub struct DisplacementMapFilter {
    pub color: Color,
    pub component_x: u8,
    pub component_y: u8,
    pub map_bitmap: Option<BitmapHandle>,
    pub map_point: (u32, u32),
    pub mode: DisplacementMapFilterMode,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Default for DisplacementMapFilter {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(0),
            component_x: 0,
            component_y: 0,
            map_bitmap: None,
            map_point: (0, 0),
            mode: DisplacementMapFilterMode::Wrap,
            scale_x: 0.0,
            scale_y: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DropShadowFilter {
    pub color: Color,
    pub angle: f32,
    pub blur_x: f32,
    pub blur_y: f32,
    pub distance: f32,
    pub hide_object: bool,
    pub inner: bool,
    pub knockout: bool,
    pub quality: u8,
    pub strength: u8,
}

impl From<swf::DropShadowFilter> for DropShadowFilter {
    fn from(value: swf::DropShadowFilter) -> Self {
        let inner = value.is_inner();
        let knockout = value.is_knockout();
        let quality = value.num_passes();
        DropShadowFilter {
            color: value.color,
            angle: value.angle.to_f32(),
            blur_x: value.blur_x.to_f32(),
            blur_y: value.blur_y.to_f32(),
            distance: value.distance.to_f32(),
            hide_object: false,
            inner,
            knockout,
            quality,
            strength: (value.strength.to_f32() * 255.0) as u8,
        }
    }
}

impl Default for DropShadowFilter {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(0),
            angle: 45.0,
            blur_x: 4.0,
            blur_y: 4.0,
            distance: 4.0,
            hide_object: false,
            inner: false,
            knockout: false,
            quality: 1,
            strength: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlowFilter {
    pub color: Color,
    pub blur_x: f32,
    pub blur_y: f32,
    pub inner: bool,
    pub knockout: bool,
    pub quality: u8,
    pub strength: u8,
}

impl From<swf::GlowFilter> for GlowFilter {
    fn from(value: swf::GlowFilter) -> Self {
        let inner = value.is_inner();
        let knockout = value.is_knockout();
        let quality = value.num_passes();
        GlowFilter {
            color: value.color,
            blur_x: value.blur_x.to_f32(),
            blur_y: value.blur_y.to_f32(),
            inner,
            knockout,
            quality,
            strength: (value.strength.to_f32() * 255.0) as u8,
        }
    }
}

impl Default for GlowFilter {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(0xFF0000FF),
            blur_x: 6.0,
            blur_y: 6.0,
            inner: false,
            knockout: false,
            quality: 1,
            strength: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GradientBevelFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: u8,
    pub bevel_type: BevelFilterType,
    pub knockout: bool,
    pub quality: u8,
}

impl From<swf::GradientFilter> for GradientBevelFilter {
    fn from(value: swf::GradientFilter) -> Self {
        let quality = value.num_passes();
        Self {
            colors: value.colors,
            blur_x: value.blur_x.to_f32(),
            blur_y: value.blur_y.to_f32(),
            angle: value.angle.to_f32(),
            distance: value.distance.to_f32(),
            strength: (value.strength.to_f32() * 255.0) as u8,
            bevel_type: if value.flags.contains(GradientFilterFlags::ON_TOP) {
                BevelFilterType::Full
            } else if value.flags.contains(GradientFilterFlags::INNER_SHADOW) {
                BevelFilterType::Inner
            } else {
                BevelFilterType::Outer
            },
            knockout: value.flags.contains(GradientFilterFlags::KNOCKOUT),
            quality,
        }
    }
}

impl Default for GradientBevelFilter {
    fn default() -> Self {
        Self {
            colors: vec![],
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
