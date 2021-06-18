//! Application Domain objects for scripts

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
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
    let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(constr));

    Ok(DomainObject(GcCell::allocate(
        activation.context.gc_context,
        DomainObjectData { base, domain },
    ))
    .into())
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

    /// Create a new object for a given domain.
    ///
    /// This function will call instance initializers. You do not need to do so
    /// yourself.
    pub fn from_domain(
        activation: &mut Activation<'_, 'gc, '_>,
        domain: Domain<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let constr = activation.avm2().classes().application_domain;
        let proto = activation.avm2().prototypes().application_domain;
        let base =
            ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(constr));
        let mut this: Object<'gc> = DomainObject(GcCell::allocate(
            activation.context.gc_context,
            DomainObjectData { base, domain },
        ))
        .into();
        this.install_instance_traits(activation, constr)?;

        constr.call_initializer(Some(this), &[], activation, Some(constr))?;

        Ok(this)
    }

    /// Create a new object for a given script's global scope.
    ///
    /// The `domain` object will serve as the scope of last resort should the
    /// global scope not have a particular name defined.
    ///
    /// This function will call instance initializers. You do not need to do so
    /// yourself.
    pub fn script_global(
        activation: &mut Activation<'_, 'gc, '_>,
        domain: Domain<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let constr = activation.avm2().classes().global;
        let proto = activation.avm2().prototypes().global;
        let base =
            ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(constr));
        let mut this: Object<'gc> = DomainObject(GcCell::allocate(
            activation.context.gc_context,
            DomainObjectData { base, domain },
        ))
        .into();
        this.install_instance_traits(activation, constr)?;

        constr.call_initializer(Some(this), &[], activation, Some(constr))?;

        Ok(this)
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

        appdomain_deriver(constr, this, activation)
    }
}
