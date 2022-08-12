//! Vector storage object

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Vector objects.
pub fn vector_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    //Because allocators are still called to build prototypes, especially for
    //the unspecialized Vector class, we have to fall back to Object when
    //getting the parameter type for our storage.
    let param_type = class
        .as_class_params()
        .flatten()
        .unwrap_or_else(|| activation.avm2().classes().object);

    Ok(VectorObject(GcCell::allocate(
        activation.context.gc_context,
        VectorObjectData {
            base,
            vector: VectorStorage::new(0, false, param_type, activation),
        },
    ))
    .into())
}

/// An Object which stores typed properties in vector storage
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct VectorObject<'gc>(GcCell<'gc, VectorObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
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
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let value_type = vector.value_type();
        let vector_class = activation.avm2().classes().vector;

        let applied_class = vector_class.apply(activation, &[value_type.into()])?;

        let mut object: Object<'gc> = VectorObject(GcCell::allocate(
            activation.context.gc_context,
            VectorObjectData {
                base: ScriptObjectData::new(applied_class),
                vector,
            },
        ))
        .into();

        object.install_instance_slots(activation);

        Ok(object)
    }
}

impl<'gc> TObject<'gc> for VectorObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();

        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    return Ok(read.vector.get(index).unwrap_or(Value::Undefined));
                }
            }
        }

        read.base.get_property_local(name, activation)
    }

    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    let type_of = self.0.read().vector.value_type();
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
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                if let Ok(index) = name.parse::<usize>() {
                    let type_of = self.0.read().vector.value_type();
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
        activation: &mut Activation<'_, 'gc, '_>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error> {
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
        _activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<u32>, Error> {
        if last_index < self.0.read().vector.length() as u32 {
            Ok(Some(last_index.saturating_add(1)))
        } else {
            Ok(None)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        if self.0.read().vector.length() as u32 >= index {
            Ok(index
                .checked_sub(1)
                .map(|index| index.into())
                .unwrap_or(Value::Undefined))
        } else {
            Ok("".into())
        }
    }

    fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        name.parse::<u32>()
            .map(|index| self.0.read().vector.length() as u32 >= index)
            .unwrap_or(false)
    }

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_vector_storage(&self) -> Option<Ref<VectorStorage<'gc>>> {
        Some(Ref::map(self.0.read(), |vod| &vod.vector))
    }

    fn as_vector_storage_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> Option<RefMut<VectorStorage<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |vod| &mut vod.vector))
    }
}
