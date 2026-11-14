//! `flash.system.SecurityDomain` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::SecurityDomainObject;
use crate::avm2::value::Value;

/// Implements `SecurityDomain.currentDomain`'s getter
pub fn get_current_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(SecurityDomainObject::new(activation).into())
}
