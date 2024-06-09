//! Vector storage object

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::avm2::Multiname;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
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
        .ok_or("Cannot convert to unparametrized Vector")?;

    Ok(VectorObject(GcCell::new(
        activation.context.gc_context,
        VectorObjectData {
            base,
            vector: VectorStorage::new(0, false, param_type, activation),
        },
    ))
    .into())
}

/// An Object which stores typed properties in vector storage
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct VectorObject<'gc>(pub GcCell<'gc, VectorObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct VectorObjectWeak<'gc>(pub GcWeakCell<'gc, VectorObjectData<'gc>>);

impl fmt::Debug for VectorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VectorObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct VectorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Vector-structured properties
    vector: VectorStorage<'gc>,
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

        let object: Object<'gc> = VectorObject(GcCell::new(
            activation.context.gc_context,
            VectorObjectData {
                base: ScriptObjectData::new(applied_class),
                vector,
            },
        ))
        .into();

        object.install_instance_slots(activation.context.gc_context);

        Ok(object)
    }
}

impl<'gc> TObject<'gc> for VectorObject<'gc> {
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
                    return read.vector.get(index, activation);
                }
            }
        }

        read.base.get_property_local(name, activation)
    }

    fn get_index_property(self, index: usize) -> Option<Value<'gc>> {
        self.0.read().vector.get_optional(index)
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    let type_of = self.0.read().vector.value_type_for_coercion(activation);
                    let value = match value.coerce_to_type(activation, type_of)? {
                        Value::Undefined => self.0.read().vector.default(activation),
                        Value::Null => self.0.read().vector.default(activation),
                        v => v,
                    };

                    self.0
                        .write(activation.context.gc_context)
                        .vector
                        .set(index, value, activation)?;

                    return Ok(());
                }
            }
        }

        let mut write = self.0.write(activation.context.gc_context);

        write.base.set_property_local(name, value, activation)
    }

    fn init_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    let type_of = self.0.read().vector.value_type_for_coercion(activation);
                    let value = match value.coerce_to_type(activation, type_of)? {
                        Value::Undefined => self.0.read().vector.default(activation),
                        Value::Null => self.0.read().vector.default(activation),
                        v => v,
                    };

                    self.0
                        .write(activation.context.gc_context)
                        .vector
                        .set(index, value, activation)?;

                    return Ok(());
                }
            }
        }

        let mut write = self.0.write(activation.context.gc_context);

        write.base.init_property_local(name, value, activation)
    }

    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        if name.contains_public_namespace()
            && name.local_name().is_some()
            && name.local_name().unwrap().parse::<usize>().is_ok()
        {
            return Ok(true);
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
                    return self.0.read().vector.is_in_range(index);
                }
            }
        }

        self.0.read().base.has_own_property(name)
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<u32>, Error<'gc>> {
        if last_index < self.0.read().vector.length() as u32 {
            Ok(Some(last_index.saturating_add(1)))
        } else {
            Ok(None)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if self.0.read().vector.length() as u32 >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Undefined))
        } else {
            Ok(Value::Undefined)
        }
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_vector_storage(&self) -> Option<Ref<VectorStorage<'gc>>> {
        Some(Ref::map(self.0.read(), |vod| &vod.vector))
    }

    fn as_vector_storage_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<VectorStorage<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |vod| &mut vod.vector))
    }
}
