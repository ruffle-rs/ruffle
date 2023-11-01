//! Object representation for NetConnection

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::net_connection::NetConnectionHandle;
use gc_arena::barrier::unlock;
use gc_arena::lock::RefLock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::cell::{Cell, Ref, RefMut};
use std::fmt;
use std::fmt::Debug;

pub fn net_connection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class).into();
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
pub struct NetConnectionObjectData<'gc> {
    base: RefLock<ScriptObjectData<'gc>>,
    #[collect(require_static)]
    handle: Cell<Option<NetConnectionHandle>>,
}

impl<'gc> TObject<'gc> for NetConnectionObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.base.borrow()
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        unlock!(Gc::write(mc, self.0), NetConnectionObjectData, base).borrow_mut()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_net_connection(self) -> Option<NetConnectionObject<'gc>> {
        Some(self)
    }
}

impl<'gc> NetConnectionObject<'gc> {
    pub fn handle(&self) -> Option<NetConnectionHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        self.0.handle.replace(handle)
    }
}

impl<'gc> Debug for NetConnectionObject<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetConnectionObject")
    }
}
