use crate::avm2::{Activation, Error, Value};

pub fn update_after_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    *activation.context.needs_render = true;
    Ok(Value::Undefined)
}
