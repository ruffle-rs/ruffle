//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance deriver that constructs AppDomain objects.
pub fn appdomain_deriver<'gc>(
    mut constr: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    let domain = scope
        .unwrap()
        .read()
        .globals()
        .as_application_domain()
        .unwrap();
    let base_proto = constr
        .get_property(
            constr,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)?;

    DomainObject::derive(
        activation.context.gc_context,
        base_proto,
        domain,
        class,
        scope,
    )
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct DomainObject<'gc>(GcCell<'gc, DomainObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct DomainObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The domain this object holds
    domain: Domain<'gc>,
}

impl<'gc> DomainObject<'gc> {
    pub fn from_domain(
        mc: MutationContext<'gc, '_>,
        base_proto: Option<Object<'gc>>,
        domain: Domain<'gc>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(base_proto, ScriptObjectClass::NoClass);

        DomainObject(GcCell::allocate(mc, DomainObjectData { base, domain })).into()
    }

    /// Construct a primitive subclass.
    pub fn derive(
        mc: MutationContext<'gc, '_>,
        base_proto: Object<'gc>,
        domain: Domain<'gc>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(DomainObject(GcCell::allocate(mc, DomainObjectData { base, domain })).into())
    }
}

impl<'gc> TObject<'gc> for DomainObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        Some(self.0.read().domain)
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let this: Object<'gc> = Object::DomainObject(*self);

        Ok(this.into())
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::DomainObject(*self);
        let parent_domain = if let Some(parent_domain) = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?
            .as_application_domain()
        {
            parent_domain
        } else {
            activation.context.avm2.global_domain()
        };

        Ok(DomainObject::from_domain(
            activation.context.gc_context,
            Some(this),
            Domain::movie_domain(activation.context.gc_context, parent_domain),
        ))
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _class: GcCell<'gc, Class<'gc>>,
        _scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::DomainObject(*self);
        Ok(DomainObject::from_domain(
            activation.context.gc_context,
            Some(this),
            activation.context.avm2.global_domain(),
        ))
    }
}
