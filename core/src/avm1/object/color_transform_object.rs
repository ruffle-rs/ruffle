use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::bitmap::color_transform_params::ColorTransformParams;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A ColorTransform
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ColorTransformObject<'gc>(GcCell<'gc, ColorTransformData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ColorTransformData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,
    params: ColorTransformParams,
}

impl fmt::Debug for ColorTransformObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ColorTransform")
            .field("redMultiplier", &this.params.red_multiplier)
            .field("greenMultiplier", &this.params.green_multiplier)
            .field("blueMultiplier", &this.params.blue_multiplier)
            .field("alphaMultiplier", &this.params.alpha_multiplier)
            .field("redOffset", &this.params.red_offset)
            .field("greenOffset", &this.params.green_offset)
            .field("blueOffset", &this.params.blue_offset)
            .field("alphaOffset", &this.params.alpha_offset)
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
                params: ColorTransformParams {
                    red_multiplier: 0.0,
                    green_multiplier: 0.0,
                    blue_multiplier: 0.0,
                    alpha_multiplier: 0.0,
                    red_offset: 0.0,
                    green_offset: 0.0,
                    blue_offset: 0.0,
                    alpha_offset: 0.0,
                },
            },
        ))
    }

    add_field_accessors!(
        [set_params, get_params, params, ColorTransformParams],
        [
            set_red_multiplier,
            get_red_multiplier,
            params.red_multiplier,
            f64
        ],
        [
            set_green_multiplier,
            get_green_multiplier,
            params.green_multiplier,
            f64
        ],
        [
            set_blue_multiplier,
            get_blue_multiplier,
            params.blue_multiplier,
            f64
        ],
        [
            set_alpha_multiplier,
            get_alpha_multiplier,
            params.alpha_multiplier,
            f64
        ],
        [set_red_offset, get_red_offset, params.red_offset, f64],
        [set_green_offset, get_green_offset, params.green_offset, f64],
        [set_blue_offset, get_blue_offset, params.blue_offset, f64],
        [set_alpha_offset, get_alpha_offset, params.alpha_offset, f64],
    );
}

impl<'gc> TObject<'gc> for ColorTransformObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_color_transform_object -> ColorTransformObject::empty_color_transform_object);
    });
}
