use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

/// Implements `StaticText.text`
pub fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this
        .as_object()
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_text())
    {
        let text = this.text(activation.context);
        return Ok(AvmString::new(activation.gc(), text).into());
    }

    Ok(Value::Undefined)
}
