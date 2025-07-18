use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ArrayObject, ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::character::Character;
use crate::tag_utils::SwfSlice;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use std::cell::{Ref, RefCell, RefMut};

/// A class instance allocator that allocates ByteArray objects.
pub fn byte_array_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let storage = if let Some((movie, id)) = activation
        .context
        .library
        .avm2_class_registry()
        .class_symbol(class.inner_class_definition())
    {
        if let Some(lib) = activation.context.library.library_for_movie(movie) {
            if let Some(Character::BinaryData(binary_data)) = lib.character_by_id(id) {
                Some(ByteArrayStorage::from_vec(
                    SwfSlice::as_ref(&binary_data).to_vec(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some(ByteArrayStorage::new())
    };

    let storage = storage.unwrap_or_else(|| {
        unreachable!("A ByteArray subclass should have ByteArray in superclass chain")
    });

    let base = ScriptObjectData::new(class);

    Ok(ByteArrayObject(Gc::new(
        activation.gc(),
        ByteArrayObjectData {
            base,
            storage: RefCell::new(storage),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ByteArrayObject<'gc>(pub Gc<'gc, ByteArrayObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ByteArrayObjectWeak<'gc>(pub GcWeak<'gc, ByteArrayObjectData<'gc>>);

impl fmt::Debug for ByteArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteArrayObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ByteArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    storage: RefCell<ByteArrayStorage>,
}

impl<'gc> ByteArrayObject<'gc> {
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc>,
        bytes: ByteArrayStorage,
    ) -> Result<ByteArrayObject<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().bytearray;
        let base = ScriptObjectData::new(class);

        let instance = ByteArrayObject(Gc::new(
            activation.gc(),
            ByteArrayObjectData {
                base,
                storage: RefCell::new(bytes),
            },
        ));

        class.call_init(instance.into(), &[], activation)?;

        Ok(instance)
    }

    fn set_element(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.0
            .storage
            .borrow_mut()
            .set(index, value.coerce_to_u32(activation)? as u8);

        Ok(())
    }

    pub fn storage(&self) -> Ref<'_, ByteArrayStorage> {
        self.0.storage.borrow()
    }
}

impl<'gc> TObject<'gc> for ByteArrayObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    return Ok(self.get_index_property(index).unwrap());
                }
            }
        }

        self.base().get_property_local(name, activation)
    }

    fn get_index_property(self, index: usize) -> Option<Value<'gc>> {
        // ByteArrays never forward to base even for out-of-bounds access.
        Some(
            self.0
                .storage
                .borrow()
                .get(index)
                .map_or(Value::Undefined, |val| Value::Integer(val as i32)),
        )
    }

    fn set_index_property(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        value: Value<'gc>,
    ) -> Option<Result<(), Error<'gc>>> {
        // ByteArrays never forward to base even for out-of-bounds access.
        Some(self.set_element(activation, index, value))
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    return self.set_element(activation, index, value);
                }
            }
        }

        self.base().set_property_local(name, value, activation)
    }

    fn init_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    return self.set_element(activation, index, value);
                }
            }
        }

        self.base().init_property_local(name, value, activation)
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    return self.0.storage.borrow().get(index).is_some();
                }
            }
        }

        self.base().has_own_property(name)
    }

    fn as_bytearray(&self) -> Option<Ref<'_, ByteArrayStorage>> {
        Some(self.0.storage.borrow())
    }

    fn as_bytearray_mut(&self) -> Option<RefMut<'_, ByteArrayStorage>> {
        Some(self.0.storage.borrow_mut())
    }

    fn as_bytearray_object(&self) -> Option<ByteArrayObject<'gc>> {
        Some(*self)
    }
}
