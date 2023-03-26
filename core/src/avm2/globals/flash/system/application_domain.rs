//! `flash.system.ApplicationDomain` class

use crate::avm2::activation::Activation;
use crate::avm2::object::{DomainObject, Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::QName;
use crate::avm2::{Domain, Error};

pub use crate::avm2::object::application_domain_allocator;

/// Implements `flash.system.ApplicationDomain`'s init method, which
/// is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        let parent_domain = if matches!(args[0], Value::Null) {
            activation.avm2().global_domain()
        } else {
            args.get_object(activation, 0, "parentDomain")?
                .as_application_domain()
                .expect("Invalid parent domain")
        };
        let fresh_domain = Domain::movie_domain(activation, parent_domain);
        this.init_application_domain(activation.context.gc_context, fresh_domain);
    }

    Ok(Value::Undefined)
}

/// `currentDomain` static property.
pub fn get_current_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation.caller_domain();

    Ok(DomainObject::from_domain(activation, appdomain)?.into())
}

/// `parentDomain` property
pub fn get_parent_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        if let Some(parent_domain) = appdomain.parent_domain() {
            return Ok(DomainObject::from_domain(activation, parent_domain)?.into());
        }
    }

    Ok(Value::Null)
}

/// `getDefinition` method
pub fn get_definition<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;
        let name = QName::from_qualified_name(name, activation);
        return appdomain.get_defined_value_handling_vector(activation, name);
    }

    Ok(Value::Undefined)
}

/// `hasDefinition` method
pub fn has_definition<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;

        let qname = QName::from_qualified_name(name, activation);

        return Ok(appdomain
            .get_defined_value_handling_vector(activation, qname)
            .is_ok()
            .into());
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property setter
pub fn set_domain_memory<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(Value::Object(arg)) = args.get(0) {
        if let Some(bytearray_obj) = arg.as_bytearray_object() {
            if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
                appdomain.set_domain_memory(activation.context.gc_context, bytearray_obj);
            }
        }
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property getter
pub fn get_domain_memory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let bytearray_object: Object<'gc> = appdomain.domain_memory().into();
        return Ok(bytearray_object.into());
    }

    Ok(Value::Undefined)
}
