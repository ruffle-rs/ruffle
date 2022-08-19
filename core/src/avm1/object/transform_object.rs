use crate::avm1::{Object, ScriptObject, TObject};
use crate::display_object::MovieClip;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A flash.geom.Transform object
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct TransformObject<'gc>(GcCell<'gc, TransformData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct TransformData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,
    clip: Option<MovieClip<'gc>>,
}

impl fmt::Debug for TransformObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("Transform")
            .field("clip", &this.clip)
            .finish()
    }
}

impl<'gc> TransformObject<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        Self(GcCell::allocate(
            gc_context,
            TransformData {
                base: ScriptObject::new(gc_context, proto),
                clip: None,
            },
        ))
    }

    pub fn clip(self) -> Option<MovieClip<'gc>> {
        self.0.read().clip
    }

    pub fn set_clip(self, gc_context: MutationContext<'gc, '_>, clip: MovieClip<'gc>) {
        self.0.write(gc_context).clip = Some(clip)
    }
}

impl<'gc> TObject<'gc> for TransformObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_transform_object -> TransformObject::empty);
    });
}
