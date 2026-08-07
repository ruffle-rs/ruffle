//! `flash.ui.Mouse` builtin

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::backend::ui::MouseCursor;
use crate::string::AvmString;

pub fn hide<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.ui.set_mouse_visible(false);
    Ok(Value::Undefined)
}

pub fn get_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // The mapping mirrors `flash.ui.MouseCursor`.
    let cursor = match activation.context.mouse_data.forced_cursor {
        None => "auto",
        Some(MouseCursor::Arrow) => "arrow",
        Some(MouseCursor::Hand) => "button",
        Some(MouseCursor::IBeam) => "ibeam",
        Some(MouseCursor::Grab) => "hand",
    };
    Ok(AvmString::new_utf8(activation.gc(), cursor).into())
}

pub fn set_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_string_non_null(activation, 0, "cursor")?;
    let forced = match &*value.to_utf8_lossy() {
        "auto" => None,
        "arrow" => Some(MouseCursor::Arrow),
        "button" => Some(MouseCursor::Hand),
        "ibeam" => Some(MouseCursor::IBeam),
        "hand" => Some(MouseCursor::Grab),
        _ => return Err(make_error_2008(activation, "cursor")),
    };
    activation.context.mouse_data.forced_cursor = forced;
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
