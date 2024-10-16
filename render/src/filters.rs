use crate::{
    bitmap::BitmapHandle,
    pixel_bender::{PixelBenderShaderArgument, PixelBenderShaderHandle},
};
use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;
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

pub trait ShaderObject: Downcast + Debug {
    fn clone_box(&self) -> Box<dyn ShaderObject>;

    fn equals(&self, other: &dyn ShaderObject) -> bool;
}
impl_downcast!(ShaderObject);

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DisplacementMapFilterMode {
    Clamp,
    Color,
    Ignore,
    Wrap,
}

#[derive(Debug, Clone, PartialEq)]
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
            viewscale_x: 1.0,
            viewscale_y: 1.0,
        }
    }
}
