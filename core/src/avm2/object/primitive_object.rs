//! Boxed primitives

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::impl_avm2_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

/// An Object which represents a primitive value of some other kind.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct PrimitiveObject<'gc>(GcCell<'gc, PrimitiveObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct PrimitiveObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The primitive value this object represents.
    primitive: Value<'gc>,
}

impl<'gc> PrimitiveObject<'gc> {
    /// Box a primitive into an object.
    pub fn from_primitive(
        primitive: Value<'gc>,
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        if let Value::Object(_) = primitive {
            return Err("Attempted to box an object as a primitive".into());
        }

        let base = ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::NoClass);

        Ok(PrimitiveObject(GcCell::allocate(
            mc,
            PrimitiveObjectData { base, primitive },
        ))
        .into())
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

        Ok(PrimitiveObject(GcCell::allocate(
            mc,
            PrimitiveObjectData {
                base,
                primitive: Value::Undefined,
            },
        ))
        .into())
    }
}

impl<'gc> TObject<'gc> for PrimitiveObject<'gc> {
    impl_avm2_custom_object!(base);

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().primitive.clone())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().primitive.clone())
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::PrimitiveObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(PrimitiveObject(GcCell::allocate(
            activation.context.gc_context,
            PrimitiveObjectData {
                base,
                primitive: Value::Undefined,
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
        let this: Object<'gc> = Object::PrimitiveObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(PrimitiveObject(GcCell::allocate(
            activation.context.gc_context,
            PrimitiveObjectData {
                base,
                primitive: Value::Undefined,
            },
        ))
        .into())
    }
}
