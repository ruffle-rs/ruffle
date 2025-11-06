//! Object representation for NetConnection

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::Error;
use crate::net_connection::NetConnectionHandle;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use std::cell::Cell;
use std::fmt;
use std::fmt::Debug;

pub fn net_connection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);
    let this: Object<'gc> = NetConnectionObject(Gc::new(
        activation.gc(),
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

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct NetConnectionObjectData<'gc> {
    base: ScriptObjectData<'gc>,

    handle: Cell<Option<NetConnectionHandle>>,
}

impl<'gc> TObject<'gc> for NetConnectionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl NetConnectionObject<'_> {
    pub fn handle(self) -> Option<NetConnectionHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        self.0.handle.replace(handle)
    }
}

impl Debug for NetConnectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetConnectionObject")
    }
}
