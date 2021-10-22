use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TDisplayObject, TObject, Value};
use crate::display_object::MovieClip;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
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
        TransformObject(GcCell::allocate(
            gc_context,
            TransformData {
                base: ScriptObject::object(gc_context, proto),
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

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let prototype = self
            .get("prototype", activation)?
            .coerce_to_object(activation);

        let clip = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)
            .as_display_object()
            .and_then(|o| o.as_movie_clip());

        if clip.is_none() {
            return Ok(Value::Undefined);
        }

        let this = prototype.create_bare_object(activation, prototype)?;
        self.construct_on_existing(activation, this, args)?;
        Ok(this.into())
    }
}
