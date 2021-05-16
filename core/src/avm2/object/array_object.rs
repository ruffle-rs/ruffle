//! Array-structured objects

use crate::avm1::AvmString;
use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::impl_avm2_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance deriver that constructs array objects.
pub fn array_deriver<'gc>(
    mut constr: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    let base_proto = constr
        .get_property(
            constr,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)?;

    ArrayObject::derive(base_proto, activation.context.gc_context, class, scope)
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
    /// Construct a fresh array.
    pub fn construct(base_proto: Object<'gc>, mc: MutationContext<'gc, '_>) -> Object<'gc> {
        let base = ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::NoClass);

        ArrayObject(GcCell::allocate(
            mc,
            ArrayObjectData {
                base,
                array: ArrayStorage::new(0),
            },
        ))
        .into()
    }

    /// Construct a primitive subclass.
    pub fn derive(
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(ArrayObject(GcCell::allocate(
            mc,
            ArrayObjectData {
                base,
                array: ArrayStorage::new(0),
            },
        ))
        .into())
    }

    /// Wrap an existing array in an object.
    pub fn from_array(
        array: ArrayStorage<'gc>,
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::NoClass);

        ArrayObject(GcCell::allocate(mc, ArrayObjectData { base, array })).into()
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    impl_avm2_custom_object!(base);

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

    fn resolve_any_trait(
        self,
        local_name: AvmString<'gc>,
    ) -> Result<Option<Namespace<'gc>>, Error> {
        self.0.read().base.resolve_any_trait(local_name)
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

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ArrayObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(ArrayObject(GcCell::allocate(
            activation.context.gc_context,
            ArrayObjectData {
                base,
                array: ArrayStorage::new(0),
            },
        ))
        .into())
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ArrayObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

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
