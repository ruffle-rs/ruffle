//! Object representation for NetStreams

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::streams::NetStream;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

pub fn netstream_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(NetStreamObject(GcCell::allocate(
        activation.context.gc_context,
        NetStreamObjectData {
            base,
            ns: NetStream::new(activation.context.gc_context),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct NetStreamObject<'gc>(GcCell<'gc, NetStreamObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NetStreamObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    ns: GcCell<'gc, NetStream>,
}

impl<'gc> TObject<'gc> for NetStreamObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_netstream(self) -> Option<NetStreamObject<'gc>> {
        Some(self)
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
