use crate::avm2::bytearray::{ByteArrayError, Endian, ObjectEncoding};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, Error};
use crate::socket::SocketHandle;
use gc_arena::{Collect, Gc};
use gc_arena::{GcWeak, Mutation};
use std::cell::{Cell, RefCell, RefMut};
use std::fmt;

/// A class instance allocator that allocates ShaderData objects.
pub fn socket_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(SocketObject(Gc::new(
        activation.context.gc(),
        SocketObjectData {
            base,
            // Default endianness is Big.
            endian: Cell::new(Endian::Big),
            object_encoding: Cell::new(ObjectEncoding::Amf3),
            timeout: Cell::new(0),
            handle: Cell::new(None),
            read_buffer: RefCell::new(vec![]),
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

    pub fn object_encoding(&self) -> ObjectEncoding {
        self.0.object_encoding.get()
    }

    pub fn set_object_encoding(&self, object_encoding: ObjectEncoding) {
        self.0.object_encoding.set(object_encoding)
    }

    pub fn timeout(&self) -> u32 {
        self.0.timeout.get()
    }

    pub fn set_timeout(&self, timeout: u32) {
        // NOTE: When a timeout of smaller than 250 milliseconds is provided,
        //       we clamp it to 250 milliseconds.
        self.0.timeout.set(std::cmp::max(250, timeout));
    }

    pub fn handle(&self) -> Option<SocketHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: SocketHandle) -> Option<SocketHandle> {
        self.0.handle.replace(Some(handle))
    }

    pub fn read_buffer(&self) -> RefMut<'_, Vec<u8>> {
        self.0.read_buffer.borrow_mut()
    }

    pub fn write_buffer(&self) -> RefMut<'_, Vec<u8>> {
        self.0.write_buffer.borrow_mut()
    }

    pub fn read_bytes(&self, amnt: usize) -> Result<Vec<u8>, ByteArrayError> {
        let mut buf = self.read_buffer();

        if amnt > buf.len() {
            return Err(ByteArrayError::EndOfFile);
        }

        // This will not panic as we have checked if we have enough bytes.
        let bytes = buf.drain(0..amnt);

        Ok(bytes.collect())
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        self.0.write_buffer.borrow_mut().extend_from_slice(bytes)
    }

    pub fn read_boolean(&self) -> Result<bool, ByteArrayError> {
        Ok(self.read_bytes(1)? != [0])
    }

    pub fn write_boolean(&self, val: bool) {
        self.write_bytes(&[val as u8; 1])
    }

    /// Same as `read_bytes`, but:
    /// - cuts the result at the first null byte to recreate a bug in FP
    /// - strips off an optional UTF8 BOM at the beginning
    pub fn read_utf_bytes(&self, amnt: usize) -> Result<Vec<u8>, ByteArrayError> {
        let mut bytes = &*self.read_bytes(amnt)?;
        if let Some(without_bom) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
            bytes = without_bom;
        }
        if let Some(null) = bytes.iter().position(|b| *b == b'\0') {
            bytes = &bytes[..null];
        }
        Ok(bytes.to_vec())
    }

    pub fn read_utf(&self) -> Result<Vec<u8>, ByteArrayError> {
        let len = self.read_unsigned_short()?;
        let val = self.read_utf_bytes(len.into())?;
        Ok(val)
    }

    // Writes a UTF String into the buffer, with its length as a prefix
    pub fn write_utf(&self, utf_string: &str) -> Result<(), Error<'gc>> {
        if let Ok(str_size) = u16::try_from(utf_string.len()) {
            self.write_unsigned_short(str_size);
            self.write_bytes(utf_string.as_bytes());
            Ok(())
        } else {
            Err("RangeError: UTF String length must fit into a short".into())
        }
    }
}

macro_rules! impl_write{
    ($($method_name:ident $data_type:ty ), *)
    =>
    {
        impl<'gc> SocketObject<'gc> {
            $( pub fn $method_name (&self, val: $data_type) {
                let val_bytes = match self.endian() {
                    Endian::Big => val.to_be_bytes(),
                    Endian::Little => val.to_le_bytes(),
                };
                self.write_bytes(&val_bytes)
             } )*
        }
    }
}

macro_rules! impl_read{
    ($($method_name:ident $size:expr; $data_type:ty ), *)
    =>
    {
        impl<'gc> SocketObject<'gc> {
            $( pub fn $method_name (&self) -> Result<$data_type, ByteArrayError> {
                Ok(match self.endian() {
                    Endian::Big => <$data_type>::from_be_bytes(self.read_bytes($size)?.try_into().unwrap()),
                    Endian::Little => <$data_type>::from_le_bytes(self.read_bytes($size)?.try_into().unwrap())
                })
             } )*
        }
    }
}

impl_write!(write_float f32, write_double f64, write_int i32, write_unsigned_int u32, write_short i16, write_unsigned_short u16);
impl_read!(read_float 4; f32, read_double 8; f64, read_int 4; i32, read_unsigned_int 4; u32, read_short 2; i16, read_unsigned_short 2; u16, read_byte 1; i8, read_unsigned_byte 1; u8);

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct SocketObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    handle: Cell<Option<SocketHandle>>,

    endian: Cell<Endian>,
    object_encoding: Cell<ObjectEncoding>,
    /// Socket connection timeout in milliseconds.
    timeout: Cell<u32>,

    read_buffer: RefCell<Vec<u8>>,
    write_buffer: RefCell<Vec<u8>>,
}

const _: () = assert!(std::mem::offset_of!(SocketObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<SocketObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl fmt::Debug for SocketObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SocketObject")
    }
}
