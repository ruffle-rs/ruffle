use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

/// A ColorMatrixFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ColorMatrixFilterObject<'gc>(GcCell<'gc, ColorMatrixFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ColorMatrixFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    matrix: [f64; 4 * 5],
}

impl fmt::Debug for ColorMatrixFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ColorMatrixFilter")
            .field("matrix", &this.matrix)
            .finish()
    }
}

impl<'gc> ColorMatrixFilterObject<'gc> {
    add_field_accessors!([set_matrix, matrix, matrix, [f64; 4 * 5]],);

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        ColorMatrixFilterObject(GcCell::allocate(
            gc_context,
            ColorMatrixFilterData {
                base: ScriptObject::object(gc_context, proto),
                matrix: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                ],
            },
        ))
    }
}

impl<'gc> TObject<'gc> for ColorMatrixFilterObject<'gc> {
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
            Some(activation.context.avm1.prototypes.color_matrix_filter),
        )
    }

    fn as_color_matrix_filter_object(&self) -> Option<ColorMatrixFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(ColorMatrixFilterObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.color_matrix_filter),
        )
        .into())
    }
}
