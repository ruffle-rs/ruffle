use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::object::Object;
use crate::avm2::value::Value;

pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;
    Ok(Value::Undefined)
}
