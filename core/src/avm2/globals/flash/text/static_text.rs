use crate::avm2::error::argument_error;
use crate::avm2::{Activation, ClassObject, Error, Object, Value};
use crate::avm2_stub_getter;

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
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.text.StaticText", "text");
    Ok("".into())
}
