use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates ByteArray objects.
pub fn bytearray_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(ByteArrayObject(GcCell::allocate(
        activation.context.gc_context,
        ByteArrayObjectData {
            base,
            storage: ByteArrayStorage::new(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ByteArrayObject<'gc>(GcCell<'gc, ByteArrayObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ByteArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    storage: ByteArrayStorage,
}

impl<'gc> ByteArrayObject<'gc> {
    pub fn from_storage(
        activation: &mut Activation<'_, 'gc, '_>,
        bytes: ByteArrayStorage,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().bytearray;
        let proto = activation.avm2().prototypes().bytearray;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut instance: Object<'gc> = ByteArrayObject(GcCell::allocate(
            activation.context.gc_context,
            ByteArrayObjectData {
                base,
                storage: bytes,
            },
        ))
        .into();
        instance.install_instance_traits(activation, class)?;

        class.call_native_init(Some(instance), &[], activation)?;

        Ok(instance)
    }
}

impl<'gc> TObject<'gc> for ByteArrayObject<'gc> {
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
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();

        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(if let Some(val) = read.storage.get(index) {
                    Value::Unsigned(val as u32)
                } else {
                    Value::Undefined
                });
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
                write
                    .storage
                    .set(index, value.coerce_to_u32(activation)? as u8);

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
                write
                    .storage
                    .set(index, value.coerce_to_u32(activation)? as u8);

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

    fn delete_property_local(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName<'gc>,
    ) -> Result<bool, Error> {
        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                self.0.write(gc_context).storage.delete(index);
                return Ok(true);
            }
        }

        Ok(self.0.write(gc_context).base.delete_property(name))
    }

    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        if name.namespace().is_public() {
            if let Ok(index) = name.local_name().parse::<usize>() {
                return Ok(self.0.read().storage.get(index).is_some());
            }
        }

        self.0.read().base.has_own_property(name)
    }

    fn resolve_ns(self, local_name: AvmString<'gc>) -> Result<Vec<Namespace<'gc>>, Error> {
        let base = self.base();

        let mut ns_set = base.resolve_ns(local_name)?;
        if !ns_set.contains(&Namespace::public()) {
            if let Ok(index) = local_name.parse::<usize>() {
                if self.0.read().storage.get(index).is_some() {
                    ns_set.push(Namespace::public())
                }
            }
        }

        Ok(ns_set)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ByteArrayObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(ByteArrayObject(GcCell::allocate(
            activation.context.gc_context,
            ByteArrayObjectData {
                base,
                storage: ByteArrayStorage::new(),
            },
        ))
        .into())
    }
    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_bytearray(&self) -> Option<Ref<ByteArrayStorage>> {
        Some(Ref::map(self.0.read(), |d| &d.storage))
    }

    fn as_bytearray_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<ByteArrayStorage>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.storage))
    }

    fn as_bytearray_object(&self) -> Option<ByteArrayObject<'gc>> {
        Some(*self)
    }
}
