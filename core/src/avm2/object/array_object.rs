//! Array-structured objects

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates array objects.
pub fn array_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(ArrayObject(Gc::new(
        activation.context.gc_context,
        ArrayObjectData {
            base,
            array: RefLock::new(ArrayStorage::new(0)),
        },
    ))
    .into())
}

/// An Object which stores numerical properties in an array.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(pub Gc<'gc, ArrayObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct ArrayObjectWeak<'gc>(pub GcWeak<'gc, ArrayObjectData<'gc>>);

impl fmt::Debug for ArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArrayObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Array-structured properties
    array: RefLock<ArrayStorage<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(ArrayObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<ArrayObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> ArrayObject<'gc> {
    /// Construct an empty array.
    pub fn empty(activation: &mut Activation<'_, 'gc>) -> Result<Object<'gc>, Error<'gc>> {
        Self::from_storage(activation, ArrayStorage::new(0))
    }

    /// Build an array object from storage.
    ///
    /// This will produce an instance of the system `Array` class.
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc>,
        array: ArrayStorage<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().array;
        let base = ScriptObjectData::new(class);

        let instance: Object<'gc> = ArrayObject(Gc::new(
            activation.context.gc_context,
            ArrayObjectData {
                base,
                array: RefLock::new(array),
            },
        ))
        .into();

        class.call_super_init(instance.into(), &[], activation)?;

        Ok(instance)
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    if let Some(result) = self.get_index_property(index) {
                        return Ok(result);
                    }
                }
            }
        }

        self.base().get_property_local(name, activation)
    }

    fn get_index_property(self, index: usize) -> Option<Value<'gc>> {
        self.0.array.borrow().get(index)
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.context.gc_context;

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    unlock!(Gc::write(mc, self.0), ArrayObjectData, array)
                        .borrow_mut()
                        .set(index, value);

                    return Ok(());
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
        let mc = activation.context.gc_context;

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    unlock!(Gc::write(mc, self.0), ArrayObjectData, array)
                        .borrow_mut()
                        .set(index, value);

                    return Ok(());
                }
            }
        }

        self.base().init_property_local(name, value, activation)
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let mc = activation.context.gc_context;

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    unlock!(Gc::write(mc, self.0), ArrayObjectData, array)
                        .borrow_mut()
                        .delete(index);

                    return Ok(true);
                }
            }
        }

        Ok(self.base().delete_property_local(mc, name))
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    return self.0.array.borrow().get(index).is_some();
                }
            }
        }

        self.base().has_own_property(name)
    }

    fn get_next_enumerant(
        self,
        mut last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<u32>, Error<'gc>> {
        let array = self.0.array.borrow();

        let array_length = array.length() as u32;

        // Array enumeration skips over holes.
        if let Some(index) = array.get_next_enumerant(last_index as usize) {
            return Ok(Some(index as u32));
        }

        last_index = std::cmp::max(last_index, array_length);

        drop(array);

        // After enumerating all of the 'normal' array entries,
        // we enumerate all of the local properties stored on the
        // ScriptObject.
        if let Some(index) = self.base().get_next_enumerant(last_index - array_length) {
            return Ok(Some(index + array_length));
        }
        Ok(None)
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let arr_len = self.0.array.borrow().length() as u32;
        if arr_len >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Undefined))
        } else {
            Ok(self
                .base()
                .get_enumerant_name(index - arr_len)
                .unwrap_or(Value::Undefined))
        }
    }

    fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        name.parse::<u32>()
            .map(|index| index < self.0.array.borrow().length() as u32)
            .unwrap_or(false)
            || self.base().property_is_enumerable(name)
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        Some(*self)
    }

    fn as_array_storage(&self) -> Option<Ref<ArrayStorage<'gc>>> {
        Some(self.0.array.borrow())
    }

    fn as_array_storage_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<ArrayStorage<'gc>>> {
        Some(unlock!(Gc::write(mc, self.0), ArrayObjectData, array).borrow_mut())
    }
}
