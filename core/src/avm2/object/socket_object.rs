use crate::avm2::bytearray::Endian;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, Error};
use crate::socket::SocketHandle;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc};
use gc_arena::{GcWeak, Mutation};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt;

/// A class instance allocator that allocates ShaderData objects.
pub fn socket_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class).into();

    Ok(SocketObject(Gc::new(
        activation.context.gc(),
        SocketObjectData {
            base,
            // Default endianness is Big.
            endian: Cell::new(Endian::Big),
            handle: Cell::new(None),
            write_buffer: RefCell::new(vec![]),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SocketObject<'gc>(pub Gc<'gc, SocketObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SocketObjectWeak<'gc>(pub GcWeak<'gc, SocketObjectData<'gc>>);

impl<'gc> TObject<'gc> for SocketObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.base.borrow()
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        unlock!(Gc::write(mc, self.0), SocketObjectData, base).borrow_mut()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_socket(&self) -> Option<SocketObject<'gc>> {
        Some(*self)
    }
}

impl<'gc> SocketObject<'gc> {
    pub fn endian(&self) -> Endian {
        self.0.endian.get()
    }

    pub fn set_endian(&self, endian: Endian) {
        self.0.endian.set(endian)
    }

    pub fn get_handle(&self) -> Option<SocketHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: SocketHandle) -> Option<SocketHandle> {
        self.0.handle.replace(Some(handle))
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        self.0.write_buffer.borrow_mut().extend_from_slice(bytes)
    }

    pub fn drain_write_buf(&self) -> Vec<u8> {
        let mut buf = self.0.write_buffer.borrow_mut();
        let len = buf.len();
        buf.drain(..len).collect::<Vec<u8>>()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct SocketObjectData<'gc> {
    /// Base script object
    base: RefLock<ScriptObjectData<'gc>>,
    #[collect(require_static)]
    handle: Cell<Option<SocketHandle>>,
    endian: Cell<Endian>,
    write_buffer: RefCell<Vec<u8>>,
}

impl fmt::Debug for SocketObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SocketObject")
    }
}
