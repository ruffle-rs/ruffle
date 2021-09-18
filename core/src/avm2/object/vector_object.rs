//! Vector storage object

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_instance};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Vector objects.
pub fn vector_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    //Because allocators are still called to build prototypes, especially for
    //the unspecialized Vector class, we have to fall back to Object when
    //getting the parameter type for our storage.
    let param_type = class
        .as_class_object_really()
        .unwrap()
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
        let applied_proto = applied_class
            .get_property(
                applied_class,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;

        let mut object: Object<'gc> = VectorObject(GcCell::allocate(
            activation.context.gc_context,
            VectorObjectData {
                base: ScriptObjectData::base_new(Some(applied_proto), Some(applied_class)),
                vector,
            },
        ))
        .into();

        object.install_instance_traits(activation, applied_class)?;

        Ok(object)
    }
}

impl<'gc> TObject<'gc> for VectorObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_instance!(base);

    fn get_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();

        if name.namespace().is_package("") {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(read.vector.get(index).unwrap_or(Value::Undefined));
            }
        }

        let rv = read.base.get_property_local(receiver, name, activation)?;

        drop(read);

        rv.resolve(activation)
    }

    fn set_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if name.namespace().is_package("") {
            if let Ok(index) = name.local_name().parse::<usize>() {
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

        let mut write = self.0.write(activation.context.gc_context);

        let rv = write
            .base
            .set_property_local(receiver, name, value, activation)?;

        drop(write);

        rv.resolve(activation)?;

        Ok(())
    }

    fn init_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if name.namespace().is_package("") {
            if let Ok(index) = name.local_name().parse::<usize>() {
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

        let mut write = self.0.write(activation.context.gc_context);

        let rv = write
            .base
            .init_property_local(receiver, name, value, activation)?;

        drop(write);

        rv.resolve(activation)?;

        Ok(())
    }

    fn is_property_overwritable(
        self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName<'gc>,
    ) -> bool {
        self.0.write(gc_context).base.is_property_overwritable(name)
    }

    fn is_property_final(self, name: &QName<'gc>) -> bool {
        self.0.read().base.is_property_final(name)
    }

    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, name: &QName<'gc>) -> bool {
        if name.namespace().is_package("") && name.local_name().parse::<usize>().is_ok() {
            return true;
        }

        self.0.write(gc_context).base.delete_property(name)
    }

    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        if name.namespace().is_package("") {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(self.0.read().vector.is_in_range(index));
            }
        }

        self.0.read().base.has_own_property(name)
    }

    fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
        if let Ok(index) = local_name.parse::<usize>() {
            if self.0.read().vector.is_in_range(index) {
                return Ok(Some(Namespace::package("")));
            }
        }

        self.0.read().base.resolve_any(local_name)
    }

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::VectorObject(*self);

        //TODO: Pull the parameter out of the class object
        let param_type = activation.avm2().classes().object;
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(VectorObject(GcCell::allocate(
            activation.context.gc_context,
            VectorObjectData {
                base,
                vector: VectorStorage::new(0, false, param_type, activation),
            },
        ))
        .into())
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
