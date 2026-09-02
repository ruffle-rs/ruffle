use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

use crate::avm2::function::FunctionArgs;
/// Implements `StaticText.text`
pub fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .as_object()
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_text())
    {
        return if let Some(text) = this.text(activation.context) {
            Ok(AvmString::new(activation.gc(), text).into())
        } else {
            Ok(Value::Null)
        };
    }

    Ok(Value::Undefined)
}
