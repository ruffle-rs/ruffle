use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

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

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Self {
        BlurFilterObject(GcCell::allocate(
            gc_context,
            BlurFilterData {
                base: ScriptObject::new(gc_context, Some(proto)),
                blur_x: 4.0,
                blur_y: 4.0,
                quality: 1,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for BlurFilterObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_blur_filter_object -> BlurFilterObject::empty_object);
    });
}
