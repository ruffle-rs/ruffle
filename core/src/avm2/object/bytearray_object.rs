use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::character::Character;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates ByteArray objects.
pub fn byte_array_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let storage = if let Some((movie, id)) = activation
        .context
        .library
        .avm2_class_registry()
        .class_symbol(class)
    {
        if let Some(lib) = activation.context.library.library_for_movie(movie) {
            if let Some(Character::BinaryData(binary_data)) = lib.character_by_id(id) {
                Some(ByteArrayStorage::from_vec(binary_data.as_ref().to_vec()))
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

    Ok(ByteArrayObject(GcCell::new(
        activation.context.gc_context,
        ByteArrayObjectData { base, storage },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ByteArrayObject<'gc>(pub GcCell<'gc, ByteArrayObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ByteArrayObjectWeak<'gc>(pub GcWeakCell<'gc, ByteArrayObjectData<'gc>>);

impl fmt::Debug for ByteArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteArrayObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ByteArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    storage: ByteArrayStorage,
}

impl<'gc> ByteArrayObject<'gc> {
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc>,
        bytes: ByteArrayStorage,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().bytearray;
        let base = ScriptObjectData::new(class);

        let instance: Object<'gc> = ByteArrayObject(GcCell::new(
            activation.context.gc_context,
            ByteArrayObjectData {
                base,
                storage: bytes,
            },
        ))
        .into();
        instance.install_instance_slots(activation.context.gc_context);

        class.call_native_init(instance.into(), &[], activation)?;

        Ok(instance)
    }

    pub fn storage(&self) -> Ref<ByteArrayStorage> {
        Ref::map(self.0.read(), |d| &d.storage)
    }
}

impl<'gc> TObject<'gc> for ByteArrayObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let read = self.0.read();

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    return Ok(self.get_index_property(index).unwrap());
                }
            }
        }

        read.base.get_property_local(name, activation)
    }

    fn get_index_property(self, index: usize) -> Option<Value<'gc>> {
        let read = self.0.read();

        // ByteArrays never forward to base even for out-of-bounds access.
        Some(
            read.storage
                .get(index)
                .map_or(Value::Undefined, |val| Value::Integer(val as i32)),
        )
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mut write = self.0.write(activation.context.gc_context);

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    write
                        .storage
                        .set(index, value.coerce_to_u32(activation)? as u8);

                    return Ok(());
                }
            }
        }

        write.base.set_property_local(name, value, activation)
    }

    fn init_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mut write = self.0.write(activation.context.gc_context);

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    write
                        .storage
                        .set(index, value.coerce_to_u32(activation)? as u8);

                    return Ok(());
                }
            }
        }

        write.base.init_property_local(name, value, activation)
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    self.0
                        .write(activation.context.gc_context)
                        .storage
                        .delete(index);
                    return Ok(true);
                }
            }
        }

        Ok(self
            .0
            .write(activation.context.gc_context)
            .base
            .delete_property_local(name))
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    return self.0.read().storage.get(index).is_some();
                }
            }
        }

        self.0.read().base.has_own_property(name)
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_bytearray(&self) -> Option<Ref<ByteArrayStorage>> {
        Some(Ref::map(self.0.read(), |d| &d.storage))
    }

    fn as_bytearray_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<ByteArrayStorage>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.storage))
    }

    fn as_bytearray_object(&self) -> Option<ByteArrayObject<'gc>> {
        Some(*self)
    }
}
