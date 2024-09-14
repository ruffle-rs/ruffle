use crate::avm1::TObject as _;
use crate::avm2::object::TObject as _;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, DisplayObjectPtr, TDisplayObject};
use gc_arena::{DynamicRoot, DynamicRootSet, Rootable};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

// TODO: Make this generic somehow
#[derive(Clone)]
pub struct DisplayObjectHandle {
    root: DynamicRoot<Rootable![DisplayObject<'_>]>,
    ptr: *const DisplayObjectPtr,
}

impl DisplayObjectHandle {
    pub fn new<'gc>(
        context: &mut UpdateContext<'gc>,
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

    pub fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.ptr
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

#[derive(Clone)]
pub struct AVM1ObjectHandle {
    root: DynamicRoot<Rootable![crate::avm1::Object<'_>]>,
    ptr: *const crate::avm1::ObjectPtr,
}

impl AVM1ObjectHandle {
    pub fn new<'gc>(context: &mut UpdateContext<'gc>, object: crate::avm1::Object<'gc>) -> Self {
        Self {
            root: context.dynamic_root.stash(context.gc_context, object),
            ptr: object.as_ptr(),
        }
    }

    pub fn fetch<'gc>(&self, dynamic_root_set: DynamicRootSet<'gc>) -> crate::avm1::Object<'gc> {
        *dynamic_root_set.fetch(&self.root)
    }
}

impl Debug for AVM1ObjectHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AVM1ObjectHandle").field(&self.ptr).finish()
    }
}

impl PartialEq<AVM1ObjectHandle> for AVM1ObjectHandle {
    #[inline(always)]
    fn eq(&self, other: &AVM1ObjectHandle) -> bool {
        self.ptr == other.ptr
    }
}

impl Hash for AVM1ObjectHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl Eq for AVM1ObjectHandle {}

#[derive(Clone)]
pub struct AVM2ObjectHandle {
    root: DynamicRoot<Rootable![crate::avm2::Object<'_>]>,
    ptr: *const crate::avm2::object::ObjectPtr,
}

impl AVM2ObjectHandle {
    pub fn new<'gc>(context: &mut UpdateContext<'gc>, object: crate::avm2::Object<'gc>) -> Self {
        Self {
            root: context.dynamic_root.stash(context.gc_context, object),
            ptr: object.as_ptr(),
        }
    }

    pub fn fetch<'gc>(&self, dynamic_root_set: DynamicRootSet<'gc>) -> crate::avm2::Object<'gc> {
        *dynamic_root_set.fetch(&self.root)
    }
}

impl Debug for AVM2ObjectHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AVM2ObjectHandle").field(&self.ptr).finish()
    }
}

impl PartialEq<AVM2ObjectHandle> for AVM2ObjectHandle {
    #[inline(always)]
    fn eq(&self, other: &AVM2ObjectHandle) -> bool {
        self.ptr == other.ptr
    }
}

impl Hash for AVM2ObjectHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl Eq for AVM2ObjectHandle {}

// Domain

#[derive(Clone)]
pub struct DomainHandle {
    root: DynamicRoot<Rootable![crate::avm2::Domain<'_>]>,
    ptr: *const crate::avm2::DomainPtr,
}

impl DomainHandle {
    pub fn new<'gc>(context: &mut UpdateContext<'gc>, domain: crate::avm2::Domain<'gc>) -> Self {
        Self {
            root: context.dynamic_root.stash(context.gc_context, domain),
            ptr: domain.as_ptr(),
        }
    }

    pub fn fetch<'gc>(&self, dynamic_root_set: DynamicRootSet<'gc>) -> crate::avm2::Domain<'gc> {
        *dynamic_root_set.fetch(&self.root)
    }
}

impl Debug for DomainHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DomainHandle").field(&self.ptr).finish()
    }
}

impl PartialEq<DomainHandle> for DomainHandle {
    #[inline(always)]
    fn eq(&self, other: &DomainHandle) -> bool {
        self.ptr == other.ptr
    }
}

impl Hash for DomainHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl Eq for DomainHandle {}
