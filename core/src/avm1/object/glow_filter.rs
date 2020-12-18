use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

/// A GlowFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct GlowFilterObject<'gc>(GcCell<'gc, GlowFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct GlowFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    alpha: f64,
    blur_x: f64,
    blur_y: f64,
    color: i32,
    inner: bool,
    knockout: bool,
    quality: i32,
    strength: f64,
}

impl fmt::Debug for GlowFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("GlowFilter")
            .field("alpha", &this.alpha)
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("color", &this.color)
            .field("inner", &this.inner)
            .field("knockout", &this.knockout)
            .field("quality", &this.quality)
            .field("strength", &this.strength)
            .finish()
    }
}

impl<'gc> GlowFilterObject<'gc> {
    add_field_accessors!(
        [set_alpha, alpha, alpha, f64],
        [set_blur_x, blur_x, blur_x, f64],
        [set_blur_y, blur_y, blur_y, f64],
        [set_color, color, color, i32],
        [set_inner, inner, inner, bool],
        [set_knockout, knockout, knockout, bool],
        [set_quality, quality, quality, i32],
        [set_strength, strength, strength, f64],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        GlowFilterObject(GcCell::allocate(
            gc_context,
            GlowFilterData {
                base: ScriptObject::object(gc_context, proto),
                alpha: 1.0,
                blur_x: 6.0,
                blur_y: 6.0,
                color: 0xFF0000,
                inner: false,
                knockout: false,
                quality: 1,
                strength: 2.0,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for GlowFilterObject<'gc> {
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
            Some(activation.context.avm1.prototypes.glow_filter),
        )
    }

    fn as_glow_filter_object(&self) -> Option<GlowFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(GlowFilterObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.glow_filter),
        )
        .into())
    }
}
