use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

/// A DropShadowFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct DropShadowFilterObject<'gc>(GcCell<'gc, DropShadowFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct DropShadowFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    alpha: f64,
    angle: f64,
    blur_x: f64,
    blur_y: f64,
    color: u32,
    distance: f64,
    hide_object: bool,
    inner: bool,
    knockout: bool,
    quality: i32,
    strength: f64,
}

impl fmt::Debug for DropShadowFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("DropShadowFilter")
            .field("alpha", &this.alpha)
            .field("angle", &this.angle)
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("color", &this.color)
            .field("distance", &this.distance)
            .field("hide_object", &this.hide_object)
            .field("inner", &this.inner)
            .field("knockout", &this.knockout)
            .field("quality", &this.quality)
            .field("strength", &this.strength)
            .finish()
    }
}

impl<'gc> DropShadowFilterObject<'gc> {
    add_field_accessors!(
        [set_alpha, alpha, alpha, f64],
        [set_angle, angle, angle, f64],
        [set_blur_x, blur_x, blur_x, f64],
        [set_blur_y, blur_y, blur_y, f64],
        [set_color, color, color, u32],
        [set_hide_object, hide_object, hide_object, bool],
        [set_distance, distance, distance, f64],
        [set_inner, inner, inner, bool],
        [set_knockout, knockout, knockout, bool],
        [set_quality, quality, quality, i32],
        [set_strength, strength, strength, f64],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        DropShadowFilterObject(GcCell::allocate(
            gc_context,
            DropShadowFilterData {
                base: ScriptObject::object(gc_context, proto),
                distance: 4.0,
                hide_object: false,
                angle: 44.9999999772279,
                color: 0x000000,
                alpha: 1.0,
                blur_x: 4.0,
                blur_y: 4.0,
                strength: 1.0,
                quality: 1,
                inner: false,
                knockout: false,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for DropShadowFilterObject<'gc> {
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
            Some(activation.context.avm1.prototypes.drop_shadow_filter),
        )
    }

    fn as_drop_shadow_filter_object(&self) -> Option<DropShadowFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(DropShadowFilterObject::empty_object(activation.context.gc_context, Some(this)).into())
    }
}
