//! Vector storage object

use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_1125, make_reference_error, Error, ReferenceErrorCode};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Multiname;
use crate::string::WStr;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Vector objects.
pub fn vector_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let param_type = class
        .inner_class_definition()
        .param()
        .expect("Class is parametrized vector");

    Ok(VectorObject(Gc::new(
        activation.gc(),
        VectorObjectData {
            base,
            vector: RefLock::new(VectorStorage::new(0, false, param_type, activation)),
        },
    ))
    .into())
}

/// An Object which stores typed properties in vector storage
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct VectorObject<'gc>(pub Gc<'gc, VectorObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct VectorObjectWeak<'gc>(pub GcWeak<'gc, VectorObjectData<'gc>>);

impl fmt::Debug for VectorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VectorObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct VectorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Vector-structured properties
    vector: RefLock<VectorStorage<'gc>>,
}

impl<'gc> VectorObject<'gc> {
    /// Wrap an existing vector in an object.
    pub fn from_vector(
        vector: VectorStorage<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let value_type = vector.value_type();
        let vector_class = activation.avm2().classes().generic_vector;

        let applied_class = vector_class.parametrize(activation, value_type)?;

        let object: Object<'gc> = VectorObject(Gc::new(
            activation.gc(),
            VectorObjectData {
                base: ScriptObjectData::new(applied_class),
                vector: RefLock::new(vector),
            },
        ))
        .into();

        Ok(object)
    }

    fn as_vector_index(local_name: &WStr) -> Option<f64> {
        // TODO: match avmplus's parsing more closely
        local_name.parse::<f64>().ok()
    }

    // Given that a read-indexing operation wasn't successful, generate an error.
    // Returns `None` if the read should fall back to the prototype chain.
    #[inline(never)]
    fn fail_read_error(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
        index: f64,
    ) -> Option<Error<'gc>> {
        // TODO the error thrown sometimes depends on JIT behavior

        if activation.caller_movie_or_root().version() >= 11 {
            // When in >=SWFv11, a RangeError is always thrown.
            let storage_len = self.0.vector.borrow().length();
            Some(make_error_1125(activation, index, storage_len))
        } else if index > 0.0 {
            // Non-negative values throw a ReferenceError on SWFv10
            Some(make_reference_error(
                activation,
                ReferenceErrorCode::InvalidRead,
                name,
                self.instance_class(),
            ))
        } else {
            // Negative values fall back to the prototype chain on SWFv10
            None
        }
    }

    // Given that a write-indexing operation wasn't successful, generate an error.
    #[inline(never)]
    fn fail_write_error(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
        index: f64,
    ) -> Error<'gc> {
        // TODO the error thrown sometimes depends on JIT behavior

        if activation.caller_movie_or_root().version() >= 11 {
            // When in >=SWFv11, a RangeError is always thrown.
            let storage_len = self.0.vector.borrow().length();
            make_error_1125(activation, index, storage_len)
        } else {
            make_reference_error(
                activation,
                ReferenceErrorCode::InvalidWrite,
                name,
                self.instance_class(),
            )
        }
    }

    fn set_element(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.gc();

        let type_of = self.0.vector.borrow().value_type_for_coercion(activation);
        let value = match value.coerce_to_type(activation, type_of)? {
            Value::Undefined => self.0.vector.borrow().default(activation),
            Value::Null => self.0.vector.borrow().default(activation),
            v => v,
        };

        unlock!(Gc::write(mc, self.0), VectorObjectData, vector)
            .borrow_mut()
            .set(index, value, activation)?;

        Ok(())
    }
}

impl<'gc> TObject<'gc> for VectorObject<'gc> {
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
            if let Some(local_name) = name.local_name() {
                if let Some(index) = VectorObject::as_vector_index(&local_name) {
                    let u32_index = index as u32;

                    if u32_index as f64 == index {
                        return self.0.vector.borrow().get(u32_index as usize, activation);
                    } else if let Some(error) = self.fail_read_error(activation, name, index) {
                        return Err(error);
                    }
                }
            }
        }

        // Now check the prototype...

        let dynamic_lookup = crate::avm2::object::get_dynamic_property(
            activation,
            name,
            None, // Vector objects have no local values
            self.proto(),
            self.instance_class(),
        )?;

        if let Some(value) = dynamic_lookup {
            Ok(value)
        } else {
            // Despite being declared dynamic, Vector classes act as if sealed
            Err(make_reference_error(
                activation,
                ReferenceErrorCode::InvalidRead,
                name,
                self.instance_class(),
            ))
        }
    }

    fn get_index_property(self, index: usize) -> Option<Value<'gc>> {
        self.0.vector.borrow().get_optional(index)
    }

    fn set_index_property(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        value: Value<'gc>,
    ) -> Option<Result<(), Error<'gc>>> {
        Some(self.set_element(activation, index, value))
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.valid_dynamic_name() {
            if let Some(local_name) = name.local_name() {
                if let Some(index) = VectorObject::as_vector_index(&local_name) {
                    let u32_index = index as u32;

                    if u32_index as f64 == index {
                        return self.set_element(activation, u32_index as usize, value);
                    } else {
                        return Err(self.fail_write_error(activation, name, index));
                    }
                }
            }
        }

        // No properties can be set on Vector classes
        Err(make_reference_error(
            activation,
            ReferenceErrorCode::InvalidWrite,
            name,
            self.instance_class(),
        ))
    }

    fn init_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.valid_dynamic_name() {
            if let Some(local_name) = name.local_name() {
                if let Some(index) = VectorObject::as_vector_index(&local_name) {
                    let u32_index = index as u32;

                    if u32_index as f64 == index {
                        return self.set_element(activation, u32_index as usize, value);
                    } else {
                        return Err(self.fail_write_error(activation, name, index));
                    }
                }
            }
        }

        // No properties can be set on Vector classes
        Err(make_reference_error(
            activation,
            ReferenceErrorCode::InvalidWrite,
            name,
            self.instance_class(),
        ))
    }

    fn delete_property_local(
        self,
        _activation: &mut Activation<'_, 'gc>,
        _name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        // FP doesn't allow deleting elements of vectors; `deleteproperty`
        // operations will always return true
        Ok(true)
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                if let Some(index) = VectorObject::as_vector_index(&name) {
                    let u32_index = index as u32;

                    if u32_index as f64 == index {
                        return self.0.vector.borrow().is_in_range(u32_index as usize);
                    } else {
                        // FIXME SWFv10 has different behavior; implementing it
                        // will require having access to an `activation` so that
                        // we can check `activation.caller_movie_or_root()`
                        return false;
                    }
                }
            }
        }

        self.base().has_own_property(name)
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<u32, Error<'gc>> {
        if last_index < self.0.vector.borrow().length() as u32 {
            Ok(last_index.saturating_add(1))
        } else {
            Ok(0)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if self.0.vector.borrow().length() as u32 >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Null))
        } else {
            Ok(Value::Null)
        }
    }

    fn as_vector_object(&self) -> Option<VectorObject<'gc>> {
        Some(*self)
    }

    fn as_vector_storage(&self) -> Option<Ref<'_, VectorStorage<'gc>>> {
        Some(self.0.vector.borrow())
    }

    fn as_vector_storage_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<'_, VectorStorage<'gc>>> {
        Some(unlock!(Gc::write(mc, self.0), VectorObjectData, vector).borrow_mut())
    }
}
