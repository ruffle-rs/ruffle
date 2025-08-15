//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::type_error;
use crate::avm2::object::{ClassObject, Object};
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn class_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Err(Error::avm_error(type_error(
        activation,
        "Error #1115: Class$ is not a constructor.",
        1115,
    )?))
}

pub fn get_prototype<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(class) = this.as_class_object() {
        return Ok(class.prototype().into());
    }

    Ok(Value::Undefined)
}
