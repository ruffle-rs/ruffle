//! Array-structured objects

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::string::{AvmString, WStr};
use crate::utils::HasPrefixField;
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
        activation.gc(),
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

#[derive(Collect, Clone, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Array-structured properties
    array: RefLock<ArrayStorage<'gc>>,
}

impl<'gc> ArrayObject<'gc> {
    /// Construct an empty array.
    pub fn empty(activation: &mut Activation<'_, 'gc>) -> ArrayObject<'gc> {
        Self::from_storage(activation, ArrayStorage::new(0))
    }

    /// Build an array object from storage.
    ///
    /// This will produce an instance of the system `Array` class.
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc>,
        array: ArrayStorage<'gc>,
    ) -> ArrayObject<'gc> {
        let class = activation.avm2().classes().array;
        let base = ScriptObjectData::new(class);

        ArrayObject(Gc::new(
            activation.gc(),
            ArrayObjectData {
                base,
                array: RefLock::new(array),
            },
        ))
    }

    pub fn as_array_index(local_name: &WStr) -> Option<usize> {
        // TODO: this should use a custom implementation instead of `parse()`,
        // see `script_object::maybe_int_property`

        local_name
            .parse::<u32>()
            .ok()
            .filter(|i| *i != u32::MAX)
            .map(|i| i as usize)
    }

    pub fn set_element(self, mc: &Mutation<'gc>, index: usize, value: Value<'gc>) {
        unlock!(Gc::write(mc, self.0), ArrayObjectData, array)
            .borrow_mut()
            .set(index, value);
    }

    pub fn array_storage(&self) -> Ref<'_, ArrayStorage<'gc>> {
        self.0.array.borrow()
    }

    pub fn array_storage_mut(&self, mc: &Mutation<'gc>) -> RefMut<'_, ArrayStorage<'gc>> {
        unlock!(Gc::write(mc, self.0), ArrayObjectData, array).borrow_mut()
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
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

    fn set_index_property(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        value: Value<'gc>,
    ) -> Option<Result<(), Error<'gc>>> {
        self.set_element(activation.gc(), index, value);

        Some(Ok(()))
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.gc();

        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    self.set_element(mc, index, value);

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
        let mc = activation.gc();

        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
                    self.set_element(mc, index, value);

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
        let mc = activation.gc();

        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
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
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = ArrayObject::as_array_index(&name) {
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
    ) -> Result<u32, Error<'gc>> {
        let array = self.0.array.borrow();

        let array_length = array.length() as u32;

        // Array enumeration skips over holes.
        if let Some(index) = array.get_next_enumerant(last_index as usize) {
            return Ok(index as u32);
        }

        last_index = std::cmp::max(last_index, array_length);

        drop(array);

        // After enumerating all of the 'normal' array entries,
        // we enumerate all of the local properties stored on the
        // ScriptObject.
        let index = self.base().get_next_enumerant(last_index - array_length);
        if index != 0 {
            return Ok(index + array_length);
        }

        Ok(0)
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
                .unwrap_or(Value::Null))
        } else {
            Ok(self
                .base()
                .get_enumerant_name(index - arr_len)
                .unwrap_or(Value::Null))
        }
    }

    fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        ArrayObject::as_array_index(&name)
            .map(|index| index < self.0.array.borrow().length())
            .unwrap_or(false)
            || self.base().property_is_enumerable(name)
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        Some(*self)
    }

    fn as_array_storage(&self) -> Option<Ref<'_, ArrayStorage<'gc>>> {
        Some(self.0.array.borrow())
    }

    fn as_array_storage_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<'_, ArrayStorage<'gc>>> {
        Some(unlock!(Gc::write(mc, self.0), ArrayObjectData, array).borrow_mut())
    }
}
