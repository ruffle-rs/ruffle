use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::{Object, ScriptObject, TObject};
use std::fmt;

/// A SharedObject
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct SharedObject<'gc>(GcCell<'gc, SharedObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SharedObjectData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    /// The local name of this shared object
    name: Option<String>,
    // In future this will also handle remote SharedObjects
}

impl fmt::Debug for SharedObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("SharedObject")
            .field("name", &this.name)
            .finish()
    }
}

impl<'gc> SharedObject<'gc> {
    pub fn empty_shared_obj(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        SharedObject(GcCell::allocate(
            gc_context,
            SharedObjectData {
                base: ScriptObject::new(gc_context, proto),
                name: None,
            },
        ))
    }

    pub fn set_name(&self, gc_context: MutationContext<'gc, '_>, name: String) {
        self.0.write(gc_context).name = Some(name);
    }

    pub fn get_name(&self) -> String {
        self.0
            .read()
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "".to_string())
    }
}

impl<'gc> TObject<'gc> for SharedObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_shared_object -> SharedObject::empty_shared_obj);
    });
}
