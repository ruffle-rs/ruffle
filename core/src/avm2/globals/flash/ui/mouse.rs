//! `flash.ui.Mouse` builtin

use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn hide<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.ui.set_mouse_visible(false);
    Ok(Value::Undefined)
}

pub fn show<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.ui.set_mouse_visible(true);
    Ok(Value::Undefined)
}
