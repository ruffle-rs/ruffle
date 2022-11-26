use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

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

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Self {
        ColorMatrixFilterObject(GcCell::allocate(
            gc_context,
            ColorMatrixFilterData {
                base: ScriptObject::new(gc_context, Some(proto)),
                matrix: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                ],
            },
        ))
    }
}

impl<'gc> TObject<'gc> for ColorMatrixFilterObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_color_matrix_filter_object -> ColorMatrixFilterObject::empty_object);
    });
}
