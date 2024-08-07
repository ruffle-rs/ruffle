//! Object representation for NetStreams

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::streams::NetStream;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::fmt::Debug;

pub fn netstream_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let ns = NetStream::new(activation.context.gc_context, None);
    let this: Object<'gc> = NetStreamObject(Gc::new(
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
pub struct NetStreamObject<'gc>(pub Gc<'gc, NetStreamObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NetStreamObjectWeak<'gc>(pub GcWeak<'gc, NetStreamObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct NetStreamObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    ns: NetStream<'gc>,
}

const _: () = assert!(std::mem::offset_of!(NetStreamObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<NetStreamObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> TObject<'gc> for NetStreamObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_netstream(self) -> Option<NetStream<'gc>> {
        Some(self.0.ns)
    }
}

impl<'gc> Debug for NetStreamObject<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("NetStreamObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
