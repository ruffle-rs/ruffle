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
use std::cell::Ref;

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
    /// Construct a
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
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    impl_avm2_custom_object!(base);

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok("function Function() {}".into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_array_storage(&self) -> Option<Ref<ArrayStorage<'gc>>> {
        Some(Ref::map(self.0.read(), |aod| &aod.array))
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
