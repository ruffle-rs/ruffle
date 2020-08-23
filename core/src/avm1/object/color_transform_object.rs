use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
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
                base: ScriptObject::object(gc_context, proto),
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
    impl_custom_object_without_set!(base);

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        let base = self.0.read().base;
        base.internal_set(
            name,
            value,
            activation,
            (*self).into(),
            Some(activation.context.avm1.prototypes.color_transform),
        )
    }

    fn as_color_transform_object(&self) -> Option<ColorTransformObject<'gc>> {
        Some(*self)
    }

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(ColorTransformObject::empty_color_transform_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.color_transform),
        )
        .into())
    }
}
