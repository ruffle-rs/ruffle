//! Object representation for NetStreams

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::Error;
use crate::streams::NetStream;
use crate::utils::HasPrefixField;
use gc_arena::{Collect, Gc, GcWeak};
use std::fmt::Debug;

pub fn netstream_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let ns = NetStream::new_avm2(activation.gc());
    let this = NetStreamObject(Gc::new(activation.gc(), NetStreamObjectData { base, ns }));

    ns.set_avm2_object(activation.gc(), this);

    ns.set_client(activation.gc(), this.into());

    Ok(this.into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct NetStreamObject<'gc>(pub Gc<'gc, NetStreamObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct NetStreamObjectWeak<'gc>(pub GcWeak<'gc, NetStreamObjectData<'gc>>);

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct NetStreamObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    ns: NetStream<'gc>,
}

impl<'gc> NetStreamObject<'gc> {
    pub fn netstream(self) -> NetStream<'gc> {
        self.0.ns
    }
}

impl<'gc> TObject<'gc> for NetStreamObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl Debug for NetStreamObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("NetStreamObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
