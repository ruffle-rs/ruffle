use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, DisplayObjectPtr, TDisplayObject};
use gc_arena::{DynamicRoot, DynamicRootSet, Rootable};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

// TODO: Make this generic somehow, we'll want AVM1 and AVM2 object handles too
#[derive(Clone)]
pub struct DisplayObjectHandle {
    root: DynamicRoot<Rootable![DisplayObject<'gc>]>,
    ptr: *const DisplayObjectPtr,
}

impl DisplayObjectHandle {
    pub fn new<'gc>(
        context: &mut UpdateContext<'_, 'gc>,
        object: impl Into<DisplayObject<'gc>>,
    ) -> Self {
        let object = object.into();
        Self {
            root: context.dynamic_root.stash(context.gc_context, object),
            ptr: object.as_ptr(),
        }
    }

    pub fn fetch<'gc>(&self, dynamic_root_set: DynamicRootSet<'gc>) -> DisplayObject<'gc> {
        *dynamic_root_set.fetch(&self.root)
    }
}

impl Debug for DisplayObjectHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DisplayObjectHandle")
            .field(&self.ptr)
            .finish()
    }
}

impl PartialEq<DisplayObjectHandle> for DisplayObjectHandle {
    #[inline(always)]
    fn eq(&self, other: &DisplayObjectHandle) -> bool {
        self.ptr == other.ptr
    }
}

impl Hash for DisplayObjectHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl Eq for DisplayObjectHandle {}
