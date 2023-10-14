//! Object representation for NetStreams

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::streams::NetStream;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

pub fn netstream_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);
    let ns = NetStream::new(activation.context.gc_context, None);
    let this: Object<'gc> = NetStreamObject(GcCell::new(
        activation.context.gc_context,
        NetStreamObjectData { base, ns },
    ))
    .into();

    ns.set_avm_object(activation.context.gc_context, this.into());

    ns.set_client(activation.context.gc_context, this);

    Ok(this)
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct NetStreamObject<'gc>(pub GcCell<'gc, NetStreamObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NetStreamObjectWeak<'gc>(pub GcWeakCell<'gc, NetStreamObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NetStreamObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    ns: NetStream<'gc>,
}

impl<'gc> TObject<'gc> for NetStreamObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_netstream(self) -> Option<NetStream<'gc>> {
        Some(self.0.read().ns)
    }
}

impl<'gc> Debug for NetStreamObject<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0.try_read() {
            Ok(obj) => f
                .debug_struct("NetStreamObject")
                .field("class", &obj.base.debug_class_name())
                .field("ptr", &self.0.as_ptr())
                .finish(),
            Err(err) => f
                .debug_struct("NetStreamObject")
                .field("class", &err)
                .field("ptr", &self.0.as_ptr())
                .finish(),
        }
    }
}
