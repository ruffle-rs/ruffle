//! `flash.system.Security` native methods

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;

pub fn get_sandbox_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let sandbox_type = activation.context.system.sandbox_type.to_string();
    return Ok(AvmString::new_utf8(activation.context.gc_context, sandbox_type).into());
}

pub fn allow_domain<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Security.allowDomain not implemented");
    Ok(Value::Undefined)
}

pub fn allow_insecure_domain<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Security.allowInsecureDomain not implemented");
    Ok(Value::Undefined)
}

pub fn load_policy_file<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Security.loadPolicyFile not implemented");
    Ok(Value::Undefined)
}

pub fn show_settings<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Security.showSettings not implemented");
    Ok(Value::Undefined)
}
