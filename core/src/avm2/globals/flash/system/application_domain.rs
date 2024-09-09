//! `flash.system.ApplicationDomain` class

use crate::avm2::activation::Activation;
use crate::avm2::object::{DomainObject, Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Domain, Error};

pub use crate::avm2::object::application_domain_allocator;

/// Implements `flash.system.ApplicationDomain`'s init method, which
/// is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let parent_domain = if matches!(args[0], Value::Null) {
        activation.avm2().playerglobals_domain()
    } else {
        args.get_object(activation, 0, "parentDomain")?
            .as_application_domain()
            .expect("Invalid parent domain")
    };
    let fresh_domain = Domain::movie_domain(activation, parent_domain);
    this.init_application_domain(activation.context.gc_context, fresh_domain);

    Ok(Value::Undefined)
}

/// `currentDomain` static property.
pub fn get_current_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation
        .caller_domain()
        .expect("Missing caller domain in ApplicationDomain.currentDomain");

    Ok(DomainObject::from_domain(activation, appdomain)?.into())
}

/// `parentDomain` property
pub fn get_parent_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        if let Some(parent_domain) = appdomain.parent_domain() {
            if parent_domain.is_playerglobals_domain(activation.avm2()) {
                return Ok(Value::Null);
            }
            return Ok(DomainObject::from_domain(activation, parent_domain)?.into());
        }
    }

    Ok(Value::Null)
}

/// `getDefinition` method
pub fn get_definition<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;
        return appdomain.get_defined_value_handling_vector(activation, name);
    }

    Ok(Value::Undefined)
}

/// `hasDefinition` method
pub fn has_definition<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;

        return Ok(appdomain
            .get_defined_value_handling_vector(activation, name)
            .is_ok()
            .into());
    }

    Ok(Value::Undefined)
}

/// 'getQualifiedDefinitionNames' method.
///
/// NOTE: Normally only available in Flash Player 11.3+.
pub fn get_qualified_definition_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        // NOTE: According to the docs of 'getQualifiedDeinitionNames',
        // it is able to throw a 'SecurityError' if "The definition belongs
        // to a domain to which the calling code does not have access."
        //
        // We do not implement this.

        let storage = VectorStorage::from_values(
            appdomain
                .get_defined_names()
                .iter()
                .filter(|name| !name.namespace().is_private())
                .map(|name| Value::String(name.to_qualified_name(activation.context.gc_context)))
                .collect(),
            false,
            Some(activation.avm2().class_defs().string),
        );

        let name_vector = VectorObject::from_vector(storage, activation)?;

        return Ok(name_vector.into());
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property setter
pub fn set_domain_memory<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        let obj = args.try_get_object(activation, 0);
        if let Some(obj) = obj {
            appdomain.set_domain_memory(activation, Some(obj.as_bytearray_object().unwrap()))?;
        } else {
            appdomain.set_domain_memory(activation, None)?;
        }
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property getter
pub fn get_domain_memory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(appdomain) = this.as_application_domain() {
        if appdomain.is_default_domain_memory() {
            return Ok(Value::Null);
        } else {
            let bytearray_object: Object<'gc> = appdomain.domain_memory().into();
            return Ok(bytearray_object.into());
        }
    }

    Ok(Value::Undefined)
}
