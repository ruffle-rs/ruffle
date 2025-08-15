//! `Boolean` impl

use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn boolean_constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bool_value = args
        .get_optional(0)
        .unwrap_or(Value::Bool(false))
        .coerce_to_boolean();

    Ok(bool_value.into())
}

pub fn call_handler<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get_optional(0)
        .unwrap_or(Value::Bool(false))
        .coerce_to_boolean()
        .into())
}
