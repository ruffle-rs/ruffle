use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, Error};
use gc_arena::barrier::unlock;
use gc_arena::lock::RefLock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::cell::{Ref, RefMut};
use std::fmt;

pub fn datagram_socket_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class).into();

    Ok(DatagramSocketObject(Gc::new(
        activation.context.gc(),
        DatagramSocketObjectData { base },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DatagramSocketObject<'gc>(pub Gc<'gc, DatagramSocketObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DatagramSocketObjectWeak<'gc>(pub GcWeak<'gc, DatagramSocketObjectData<'gc>>);

impl<'gc> TObject<'gc> for DatagramSocketObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.base.borrow()
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        unlock!(Gc::write(mc, self.0), DatagramSocketObjectData, base).borrow_mut()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_datagram_socket(&self) -> Option<DatagramSocketObject<'gc>> {
        Some(*self)
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct DatagramSocketObjectData<'gc> {
    base: RefLock<ScriptObjectData<'gc>>,
}

impl fmt::Debug for DatagramSocketObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatagramSocketObject")
    }
}
