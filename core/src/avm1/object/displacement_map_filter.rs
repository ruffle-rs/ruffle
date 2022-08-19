use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use crate::string::WStr;
use gc_arena::{Collect, GcCell, MutationContext};

use std::fmt;

#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub enum DisplacementMapFilterMode {
    Wrap,
    Clamp,
    Ignore,
    Color,
}

impl<'a> From<&'a WStr> for DisplacementMapFilterMode {
    fn from(v: &'a WStr) -> DisplacementMapFilterMode {
        if v == b"clamp" {
            DisplacementMapFilterMode::Clamp
        } else if v == b"ignore" {
            DisplacementMapFilterMode::Ignore
        } else if v == b"color" {
            DisplacementMapFilterMode::Color
        } else {
            DisplacementMapFilterMode::Wrap
        }
    }
}

impl From<DisplacementMapFilterMode> for &'static WStr {
    fn from(v: DisplacementMapFilterMode) -> Self {
        let s: &[u8] = match v {
            DisplacementMapFilterMode::Wrap => b"wrap",
            DisplacementMapFilterMode::Clamp => b"clamp",
            DisplacementMapFilterMode::Ignore => b"ignore",
            DisplacementMapFilterMode::Color => b"color",
        };
        WStr::from_units(s)
    }
}

/// A DisplacementMapFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct DisplacementMapFilterObject<'gc>(GcCell<'gc, DisplacementMapFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct DisplacementMapFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    alpha: f64,
    color: u32,
    component_x: i32,
    component_y: i32,
    map_bitmap: Option<Object<'gc>>,
    map_point: (i32, i32),
    mode: DisplacementMapFilterMode,
    scale_x: f64,
    scale_y: f64,
}

impl fmt::Debug for DisplacementMapFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("DisplacementMapFilter")
            .field("alpha", &this.alpha)
            .field("color", &this.color)
            .field("componentX", &this.component_x)
            .field("componentY", &this.component_y)
            .field("mapBitmap", &this.map_bitmap)
            .field("mapPoint", &this.map_point)
            .field("mode", &this.mode)
            .field("scaleX", &this.scale_x)
            .field("scaleY", &this.scale_y)
            .finish()
    }
}

impl<'gc> DisplacementMapFilterObject<'gc> {
    add_field_accessors!(
        [set_alpha, alpha, alpha, f64],
        [set_color, color, color, u32],
        [set_component_x, component_x, component_x, i32],
        [set_component_y, component_y, component_y, i32],
        [set_map_bitmap, map_bitmap, map_bitmap, Option<Object<'gc>>],
        [set_map_point, map_point, map_point, (i32, i32)],
        [set_mode, mode, mode, DisplacementMapFilterMode],
        [set_scale_x, scale_x, scale_x, f64],
        [set_scale_y, scale_y, scale_y, f64],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        DisplacementMapFilterObject(GcCell::allocate(
            gc_context,
            DisplacementMapFilterData {
                base: ScriptObject::new(gc_context, proto),
                alpha: 0.0,
                color: 0,
                component_x: 0,
                component_y: 0,
                map_bitmap: None,
                map_point: (0, 0),
                mode: DisplacementMapFilterMode::Wrap,
                scale_x: 0.0,
                scale_y: 0.0,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for DisplacementMapFilterObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_displacement_map_filter_object -> DisplacementMapFilterObject::empty_object);
    });
}
