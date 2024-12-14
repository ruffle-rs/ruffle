//! Object representation for NetConnection

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::Error;
use crate::net_connection::NetConnectionHandle;
use gc_arena::{Collect, Gc, GcWeak};
use std::cell::Cell;
use std::fmt;
use std::fmt::Debug;

pub fn net_connection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);
    let this: Object<'gc> = NetConnectionObject(Gc::new(
        activation.context.gc_context,
        NetConnectionObjectData {
            base,
            handle: Cell::new(None),
        },
    ))
    .into();

    Ok(this)
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct NetConnectionObject<'gc>(pub Gc<'gc, NetConnectionObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NetConnectionObjectWeak<'gc>(pub GcWeak<'gc, NetConnectionObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct NetConnectionObjectData<'gc> {
    base: ScriptObjectData<'gc>,

    handle: Cell<Option<NetConnectionHandle>>,
}

const _: () = assert!(std::mem::offset_of!(NetConnectionObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<NetConnectionObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> TObject<'gc> for NetConnectionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_net_connection(self) -> Option<NetConnectionObject<'gc>> {
        Some(self)
    }
}

impl NetConnectionObject<'_> {
    pub fn handle(&self) -> Option<NetConnectionHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        self.0.handle.replace(handle)
    }
}

impl Debug for NetConnectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetConnectionObject")
    }
}
