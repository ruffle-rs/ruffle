use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use crate::string::WStr;
use gc_arena::{Collect, GcCell, MutationContext};

use std::fmt;

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum BevelFilterType {
    Inner,
    Outer,
    Full,
}

impl<'a> From<&'a WStr> for BevelFilterType {
    fn from(value: &'a WStr) -> Self {
        if value == b"inner" {
            BevelFilterType::Inner
        } else if value == b"outer" {
            BevelFilterType::Outer
        } else {
            BevelFilterType::Full
        }
    }
}

impl From<BevelFilterType> for &'static WStr {
    fn from(v: BevelFilterType) -> &'static WStr {
        let s: &[u8] = match v {
            BevelFilterType::Inner => b"inner",
            BevelFilterType::Outer => b"outer",
            BevelFilterType::Full => b"full",
        };
        WStr::from_units(s)
    }
}

/// A BevelFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BevelFilterObject<'gc>(GcCell<'gc, BevelFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BevelFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    angle: f64,
    blur_x: f64,
    blur_y: f64,
    distance: f64,
    highlight_alpha: f64,
    highlight_color: u32,
    knockout: bool,
    quality: i32,
    shadow_alpha: f64,
    shadow_color: u32,
    strength: f64,
    type_: BevelFilterType,
}

impl fmt::Debug for BevelFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BevelFilter")
            .field("angle", &this.angle)
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("distance", &this.distance)
            .field("highlightAlpha", &this.highlight_alpha)
            .field("highlightColor", &this.highlight_color)
            .field("knockout", &this.knockout)
            .field("quality", &this.quality)
            .field("shadowAlpha", &this.shadow_alpha)
            .field("strength", &this.strength)
            .field("type", &this.type_)
            .finish()
    }
}

impl<'gc> BevelFilterObject<'gc> {
    add_field_accessors!(
        [set_angle, angle, angle, f64],
        [set_blur_x, blur_x, blur_x, f64],
        [set_blur_y, blur_y, blur_y, f64],
        [set_distance, distance, distance, f64],
        [set_highlight_alpha, highlight_alpha, highlight_alpha, f64],
        [set_highlight_color, highlight_color, highlight_color, u32],
        [set_knockout, knockout, knockout, bool],
        [set_quality, quality, quality, i32],
        [set_shadow_alpha, shadow_alpha, shadow_alpha, f64],
        [set_shadow_color, shadow_color, shadow_color, u32],
        [set_strength, strength, strength, f64],
        [set_type, get_type, type_, BevelFilterType],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Self {
        BevelFilterObject(GcCell::allocate(
            gc_context,
            BevelFilterData {
                base: ScriptObject::new(gc_context, Some(proto)),
                angle: 44.9999999772279,
                blur_x: 4.0,
                blur_y: 4.0,
                distance: 4.0,
                highlight_alpha: 1.0,
                highlight_color: 0xFFFFFF,
                knockout: false,
                quality: 1,
                shadow_alpha: 1.0,
                shadow_color: 0x000000,
                strength: 1.0,
                type_: BevelFilterType::Inner,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for BevelFilterObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_bevel_filter_object -> BevelFilterObject::empty_object);
    });
}
