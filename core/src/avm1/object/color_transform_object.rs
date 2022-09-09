use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::color_transform::ColorTransform;
use std::fmt;
use swf::Fixed8;

/// A ColorTransform
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ColorTransformObject<'gc>(GcCell<'gc, ColorTransformData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ColorTransformData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    red_multiplier: f64,
    green_multiplier: f64,
    blue_multiplier: f64,
    alpha_multiplier: f64,
    red_offset: f64,
    green_offset: f64,
    blue_offset: f64,
    alpha_offset: f64,
}

impl fmt::Debug for ColorTransformObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ColorTransform")
            .field("redMultiplier", &this.red_multiplier)
            .field("greenMultiplier", &this.green_multiplier)
            .field("blueMultiplier", &this.blue_multiplier)
            .field("alphaMultiplier", &this.alpha_multiplier)
            .field("redOffset", &this.red_offset)
            .field("greenOffset", &this.green_offset)
            .field("blueOffset", &this.blue_offset)
            .field("alphaOffset", &this.alpha_offset)
            .finish()
    }
}

impl<'gc> ColorTransformObject<'gc> {
    pub fn empty_color_transform_object(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        ColorTransformObject(GcCell::allocate(
            gc_context,
            ColorTransformData {
                base: ScriptObject::new(gc_context, proto),
                red_multiplier: 0.0,
                green_multiplier: 0.0,
                blue_multiplier: 0.0,
                alpha_multiplier: 0.0,
                red_offset: 0.0,
                green_offset: 0.0,
                blue_offset: 0.0,
                alpha_offset: 0.0,
            },
        ))
    }

    add_field_accessors!(
        [set_red_multiplier, get_red_multiplier, red_multiplier, f64],
        [
            set_green_multiplier,
            get_green_multiplier,
            green_multiplier,
            f64
        ],
        [
            set_blue_multiplier,
            get_blue_multiplier,
            blue_multiplier,
            f64
        ],
        [
            set_alpha_multiplier,
            get_alpha_multiplier,
            alpha_multiplier,
            f64
        ],
        [set_red_offset, get_red_offset, red_offset, f64],
        [set_green_offset, get_green_offset, green_offset, f64],
        [set_blue_offset, get_blue_offset, blue_offset, f64],
        [set_alpha_offset, get_alpha_offset, alpha_offset, f64],
    );
}

impl<'gc> TObject<'gc> for ColorTransformObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_color_transform_object -> ColorTransformObject::empty_color_transform_object);
    });
}

impl From<ColorTransformObject<'_>> for ColorTransform {
    fn from(object: ColorTransformObject) -> Self {
        Self {
            r_mult: Fixed8::from_f64(object.get_red_multiplier()),
            g_mult: Fixed8::from_f64(object.get_green_multiplier()),
            b_mult: Fixed8::from_f64(object.get_blue_multiplier()),
            a_mult: Fixed8::from_f64(object.get_alpha_multiplier()),
            r_add: object.get_red_offset() as i16,
            g_add: object.get_green_offset() as i16,
            b_add: object.get_blue_offset() as i16,
            a_add: object.get_alpha_offset() as i16,
        }
    }
}
