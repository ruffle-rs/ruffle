use crate::bitmap::BitmapHandle;
use crate::pixel_bender::PixelBenderShaderHandle;
use crate::pixel_bender_support::PixelBenderShaderArgument;
use std::{any::Any, fmt::Debug};
use swf::{Color, Rectangle, Twips};

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    BevelFilter(swf::BevelFilter),
    BlurFilter(swf::BlurFilter),
    ColorMatrixFilter(swf::ColorMatrixFilter),
    ConvolutionFilter(swf::ConvolutionFilter),
    DisplacementMapFilter(DisplacementMapFilter),
    DropShadowFilter(swf::DropShadowFilter),
    GlowFilter(swf::GlowFilter),
    GradientBevelFilter(swf::GradientFilter),
    GradientGlowFilter(swf::GradientFilter),
    ShaderFilter(ShaderFilter<'static>),
}

#[derive(Debug, Clone)]
pub struct ShaderFilter<'a> {
    pub bottom_extension: i32,
    pub left_extension: i32,
    pub right_extension: i32,
    pub top_extension: i32,
    /// The AVM2 `flash.display.Shader` object that we extracted
    /// the `shader` and `shader_args` fields from. This is used when
    /// we reconstruct a `ShaderFilter` object in the AVM2 `DisplayObject.filters`
    /// (Flash re-uses the same object)
    pub shader_object: Box<dyn ShaderObject>,
    pub shader: PixelBenderShaderHandle,
    pub shader_args: Vec<PixelBenderShaderArgument<'a>>,
}

impl PartialEq for ShaderFilter<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bottom_extension == other.bottom_extension
            && self.left_extension == other.left_extension
            && self.right_extension == other.right_extension
            && self.top_extension == other.top_extension
            && self.shader_object.equals(other.shader_object.as_ref())
            && self.shader == other.shader
            && self.shader_args == other.shader_args
    }
}

pub trait ShaderObject: Any + Debug {
    fn clone_box(&self) -> Box<dyn ShaderObject>;

    fn equals(&self, other: &dyn ShaderObject) -> bool;
}

impl Clone for Box<dyn ShaderObject> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Filter {
    pub fn scale(&mut self, x: f32, y: f32) {
        match self {
            Filter::BevelFilter(filter) => filter.scale(x, y),
            Filter::BlurFilter(filter) => filter.scale(x, y),
            Filter::DropShadowFilter(filter) => filter.scale(x, y),
            Filter::GlowFilter(filter) => filter.scale(x, y),
            Filter::GradientBevelFilter(filter) => filter.scale(x, y),
            Filter::GradientGlowFilter(filter) => filter.scale(x, y),
            Filter::DisplacementMapFilter(filter) => filter.scale(x, y),
            _ => {}
        }
    }

    pub fn calculate_dest_rect(&self, source_rect: Rectangle<Twips>) -> Rectangle<Twips> {
        match self {
            Filter::BlurFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::GlowFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::DropShadowFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::BevelFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::DisplacementMapFilter(filter) => filter.calculate_dest_rect(source_rect),
            _ => source_rect,
        }
    }

    /// Calculates how far this filter's output extends beyond the source rect
    /// in `BitmapData` operations, in whole pixels.
    ///
    /// `BitmapData.generateFilterRect` reports this expansion and
    /// `BitmapData.applyFilter` writes it, so both must use this method.
    ///
    /// Unlike `calculate_dest_rect` (used for display object filter bounds,
    /// where over-estimating is harmless), these margins define exactly which
    /// destination pixels get replaced, so they follow Flash Player's values:
    /// a blur with strength `b` and `q` passes reaches `ceil((b - 1) * q / 2)`
    /// pixels per side (verified against Flash Player captures: blur 4
    /// quality 1 writes exactly 2 pixels beyond the source rect).
    pub fn calculate_dest_margins(&self) -> FilterMargins {
        match self {
            Filter::BlurFilter(filter) => FilterMargins::from_blur(
                filter.blur_x.to_f64(),
                filter.blur_y.to_f64(),
                filter.num_passes(),
            ),
            Filter::GlowFilter(filter) => {
                if filter.is_inner() {
                    FilterMargins::default()
                } else {
                    FilterMargins::from_blur(
                        filter.blur_x.to_f64(),
                        filter.blur_y.to_f64(),
                        filter.num_passes(),
                    )
                }
            }
            Filter::DropShadowFilter(filter) => {
                if filter.is_inner() {
                    FilterMargins::default()
                } else {
                    let distance = filter.distance.to_f64();
                    let angle = filter.angle.to_f64();
                    FilterMargins::from_blur(
                        filter.blur_x.to_f64(),
                        filter.blur_y.to_f64(),
                        filter.num_passes(),
                    )
                    .with_offset(angle.cos() * distance, angle.sin() * distance)
                }
            }
            Filter::BevelFilter(filter) => {
                let distance = filter.distance.to_f64();
                let angle = filter.angle.to_f64();
                // Bevel draws highlight and shadow on opposite sides.
                FilterMargins::from_blur(
                    filter.blur_x.to_f64(),
                    filter.blur_y.to_f64(),
                    filter.num_passes(),
                )
                .with_offset(angle.cos() * distance, angle.sin() * distance)
                .with_offset(-angle.cos() * distance, -angle.sin() * distance)
            }
            _ => FilterMargins::default(),
        }
    }

    /// Checks if this filter is impotent.
    /// Impotent filters will have no effect if applied, and can safely be skipped.
    pub fn impotent(&self) -> bool {
        // TODO: There's more cases here, find them!
        match self {
            Filter::BlurFilter(filter) => filter.impotent(),
            Filter::ColorMatrixFilter(filter) => filter.impotent(),
            _ => false,
        }
    }
}

/// How far a filter's output extends beyond the source image
/// on each edge, in pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FilterMargins {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl FilterMargins {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The per-side reach of a box blur with the given strength and passes,
    /// matching Flash Player's `generateFilterRect` values.
    fn from_blur(blur_x: f64, blur_y: f64, passes: u8) -> Self {
        fn axis(blur: f64, passes: u8) -> u32 {
            if blur <= 1.0 || passes == 0 {
                return 0;
            }
            (((blur - 1.0) * passes as f64) / 2.0).ceil() as u32
        }
        let x = axis(blur_x, passes);
        let y = axis(blur_y, passes);
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    /// Extends the margins on the side(s) an offset vector points towards
    /// (e.g. a drop shadow's distance/angle displacement).
    fn with_offset(mut self, dx: f64, dy: f64) -> Self {
        // Strip float noise (cos(PI/2) is not exactly zero, near-integer
        // products can land just above the integer) so that axis-aligned
        // offsets neither leak a spurious margin nor round an extra pixel.
        const EPSILON: f64 = 1e-6;
        if dx > EPSILON {
            self.right += (dx - EPSILON).ceil() as u32;
        } else if dx < -EPSILON {
            self.left += (-dx - EPSILON).ceil() as u32;
        }
        if dy > EPSILON {
            self.bottom += (dy - EPSILON).ceil() as u32;
        } else if dy < -EPSILON {
            self.top += (-dy - EPSILON).ceil() as u32;
        }
        self
    }
}

impl From<&swf::Filter> for Filter {
    fn from(value: &swf::Filter) -> Self {
        match value {
            swf::Filter::DropShadowFilter(filter) => {
                Filter::DropShadowFilter(filter.as_ref().to_owned())
            }
            swf::Filter::BlurFilter(filter) => Filter::BlurFilter(filter.as_ref().to_owned()),
            swf::Filter::GlowFilter(filter) => Filter::GlowFilter(filter.as_ref().to_owned()),
            swf::Filter::BevelFilter(filter) => Filter::BevelFilter(filter.as_ref().to_owned()),
            swf::Filter::GradientGlowFilter(filter) => {
                Filter::GradientGlowFilter(filter.as_ref().to_owned())
            }
            swf::Filter::ConvolutionFilter(filter) => {
                Filter::ConvolutionFilter(filter.as_ref().to_owned())
            }
            swf::Filter::ColorMatrixFilter(filter) => {
                Filter::ColorMatrixFilter(filter.as_ref().to_owned())
            }
            swf::Filter::GradientBevelFilter(filter) => {
                Filter::GradientBevelFilter(filter.as_ref().to_owned())
            }
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        // A default colormatrix is a filter that essentially does nothing,
        // making it a useful default in situations that we need a dummy filter
        Filter::ColorMatrixFilter(swf::ColorMatrixFilter::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DisplacementMapFilterComponent {
    Alpha,
    Blue,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum DisplacementMapFilterMode {
    Clamp,
    Color,
    Ignore,
    #[default]
    Wrap,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplacementMapFilter {
    pub color: Color,
    pub component_x: u8,
    pub component_y: u8,
    pub map_bitmap: Option<BitmapHandle>,
    pub map_point: (i32, i32),
    pub mode: DisplacementMapFilterMode,
    pub scale_x: f32,
    pub scale_y: f32,
    pub viewscale_x: f32,
    pub viewscale_y: f32,
}

impl DisplacementMapFilter {
    pub fn scale(&mut self, x: f32, y: f32) {
        self.viewscale_x *= x;
        self.viewscale_y *= y;
    }

    pub fn calculate_dest_rect(&self, source_rect: Rectangle<Twips>) -> Rectangle<Twips> {
        source_rect
        // [NA] TODO: This *appears* to be correct, but I'm not entirely sure why Flash does this.
        // This is commented out for now because Flash actually might need us to resize the texture *after* we make it,
        // which is unsupported in our current architecture as of time of writing.

        // if filter.mode == DisplacementMapFilterMode::Color {
        //     Rectangle {
        //         x_min: source_rect.x_min - ((filter.scale_x / 2.0).floor() as i32),
        //         x_max: source_rect.x_max + (filter.scale_x.floor() as i32),
        //         y_min: source_rect.y_min - ((filter.scale_y / 2.0).floor() as i32),
        //         y_max: source_rect.y_max + (filter.scale_y.floor() as i32),
        //     }
        // } else {
        //     source_rect
        // }
    }
}
