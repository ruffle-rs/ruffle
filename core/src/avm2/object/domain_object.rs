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
    constr: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let scope = constr
        .get_scope()
        .ok_or("Constructor has an empty scope stack")?;
    let domain = scope
        .read()
        .globals()
        .as_application_domain()
        .ok_or("Constructor scope must have an appdomain at the bottom of it's scope stack")?;

    DomainObject::derive(constr, proto, domain, activation.context.gc_context)
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
    /// Create a new domain without association with any class or prototype.
    ///
    /// This should only be called during early player runtime initialization.
    /// It will return a `Domain` with no proto or instance constructor link,
    /// meaning that you will have to set those yourself.
    pub fn from_early_domain(mc: MutationContext<'gc, '_>, domain: Domain<'gc>) -> Object<'gc> {
        let base = ScriptObjectData::base_new(None, ScriptObjectClass::NoClass);

        DomainObject(GcCell::allocate(mc, DomainObjectData { base, domain })).into()
    }

    pub fn from_domain(
        mc: MutationContext<'gc, '_>,
        constr: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        domain: Domain<'gc>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(base_proto, ScriptObjectClass::ClassInstance(constr));

        DomainObject(GcCell::allocate(mc, DomainObjectData { base, domain })).into()
    }

    /// Construct a primitive subclass.
    pub fn derive(
        constr: Object<'gc>,
        base_proto: Object<'gc>,
        domain: Domain<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let base =
            ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::ClassInstance(constr));

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

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let mut this: Object<'gc> = Object::DomainObject(*self);
        let constr = this
            .get_property(
                this,
                &QName::new(Namespace::public(), "constructor"),
                activation,
            )?
            .coerce_to_object(activation)?;

        Ok(DomainObject::from_domain(
            activation.context.gc_context,
            constr,
            Some(this),
            activation.context.avm2.global_domain(),
        ))
    }
}
