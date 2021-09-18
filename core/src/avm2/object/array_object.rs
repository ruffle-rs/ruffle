//! Array-structured objects

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_instance};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates array objects.
pub fn array_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(ArrayObject(GcCell::allocate(
        activation.context.gc_context,
        ArrayObjectData {
            base,
            array: ArrayStorage::new(0),
        },
    ))
    .into())
}

/// An Object which stores numerical properties in an array.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(GcCell<'gc, ArrayObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct ArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Array-structured properties
    array: ArrayStorage<'gc>,
}

impl<'gc> ArrayObject<'gc> {
    /// Construct an empty array.
    pub fn empty(activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        Self::from_storage(activation, ArrayStorage::new(0))
    }

    /// Build an array object from storage.
    ///
    /// This will produce an instance of the system `Array` class.
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc, '_>,
        array: ArrayStorage<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().array;
        let proto = activation.avm2().prototypes().array;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut instance: Object<'gc> = ArrayObject(GcCell::allocate(
            activation.context.gc_context,
            ArrayObjectData { base, array },
        ))
        .into();
        instance.install_instance_traits(activation, class)?;

        class.call_native_init(Some(instance), &[], activation, Some(class))?;

        Ok(instance)
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_instance!(base);

    fn get_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();

        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(read.array.get(index).unwrap_or(Value::Undefined));
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
        let mut write = self.0.write(activation.context.gc_context);

        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                write.array.set(index, value);

                return Ok(());
            }
        }

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
        let mut write = self.0.write(activation.context.gc_context);

        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                write.array.set(index, value);

                return Ok(());
            }
        }

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
        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                self.0.write(gc_context).array.delete(index);
                return true;
            }
        }

        self.0.write(gc_context).base.delete_property(name)
    }

    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(self.0.read().array.get(index).is_some());
            }
        }

        self.0.read().base.has_own_property(name)
    }

    fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
        if let Ok(index) = local_name.parse::<usize>() {
            if self.0.read().array.get(index).is_some() {
                return Ok(Some(Namespace::public()));
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

    fn as_array_storage(&self) -> Option<Ref<ArrayStorage<'gc>>> {
        Some(Ref::map(self.0.read(), |aod| &aod.array))
    }

    fn as_array_storage_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> Option<RefMut<ArrayStorage<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |aod| &mut aod.array))
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ArrayObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(ArrayObject(GcCell::allocate(
            activation.context.gc_context,
            ArrayObjectData {
                base,
                array: ArrayStorage::new(0),
            },
        ))
        .into())
    }
}
