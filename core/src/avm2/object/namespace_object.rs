//! Boxed namespaces

use crate::avm1::AvmString;
use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::Ref;

/// A class instance deriver that constructs namespace objects.
pub fn namespace_deriver<'gc>(
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

    NamespaceObject::derive(base_proto, activation.context.gc_context, class, scope)
}

/// An Object which represents a boxed namespace name.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct NamespaceObject<'gc>(GcCell<'gc, NamespaceObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct NamespaceObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The namespace name this object is associated with.
    namespace: Namespace<'gc>,
}

impl<'gc> NamespaceObject<'gc> {
    /// Box a namespace into an object.
    pub fn from_namespace(
        namespace: Namespace<'gc>,
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::NoClass);

        Ok(NamespaceObject(GcCell::allocate(
            mc,
            NamespaceObjectData { base, namespace },
        ))
        .into())
    }

    /// Construct a namespace subclass.
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

        Ok(NamespaceObject(GcCell::allocate(
            mc,
            NamespaceObjectData {
                base,
                namespace: Namespace::public(),
            },
        ))
        .into())
    }
}

impl<'gc> TObject<'gc> for NamespaceObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().namespace.as_uri().into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().namespace.as_uri().into())
    }

    fn as_namespace(&self) -> Option<Ref<Namespace<'gc>>> {
        Some(Ref::map(self.0.read(), |s| &s.namespace))
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::NamespaceObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(NamespaceObject(GcCell::allocate(
            activation.context.gc_context,
            NamespaceObjectData {
                base,
                namespace: Namespace::public(),
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
        let this: Object<'gc> = Object::NamespaceObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(NamespaceObject(GcCell::allocate(
            activation.context.gc_context,
            NamespaceObjectData {
                base,
                namespace: Namespace::public(),
            },
        ))
        .into())
    }
}
