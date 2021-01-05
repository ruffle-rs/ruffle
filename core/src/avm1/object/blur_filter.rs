use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

/// A BlurFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BlurFilterObject<'gc>(GcCell<'gc, BlurFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BlurFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    blur_x: f64,
    blur_y: f64,
    quality: i32,
}

impl fmt::Debug for BlurFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BlurFilter")
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("quality", &this.quality)
            .finish()
    }
}

impl<'gc> BlurFilterObject<'gc> {
    add_field_accessors!(
        [set_blur_x, blur_x, blur_x, f64],
        [set_blur_y, blur_y, blur_y, f64],
        [set_quality, quality, quality, i32],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        BlurFilterObject(GcCell::allocate(
            gc_context,
            BlurFilterData {
                base: ScriptObject::object(gc_context, proto),
                blur_x: 4.0,
                blur_y: 4.0,
                quality: 1,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for BlurFilterObject<'gc> {
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
            Some(activation.context.avm1.prototypes.blur_filter),
        )
    }

    fn as_blur_filter_object(&self) -> Option<BlurFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(BlurFilterObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.blur_filter),
        )
        .into())
    }
}
