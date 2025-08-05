//! `flash.system.SecurityDomain` native methods

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2012;
use crate::avm2::object::SecurityDomainObject;
use crate::avm2::{ClassObject, Error, Object, Value};

pub fn security_domain_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Err(make_error_2012(activation, "SecurityDomain"))
}

pub fn instantiate_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(SecurityDomainObject::new(activation).into())
}
