use crate::avm2::error::argument_error;
use crate::avm2::{Activation, ClassObject, Error, Object, TObject, Value};
use crate::prelude::TDisplayObject;
use crate::string::AvmString;

pub fn static_text_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    return Err(Error::AvmError(argument_error(
        activation,
        "Error #2012: StaticText$ class cannot be instantiated.",
        2012,
    )?));
}

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
