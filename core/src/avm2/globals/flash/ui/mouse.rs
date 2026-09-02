//! `flash.ui.Mouse` builtin

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Avm2StrRepresentable, Error};
use crate::backend::ui::MouseCursor;
use ruffle_macros::istr;

pub fn hide<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.ui.set_mouse_visible(false);
    Ok(Value::Undefined)
}

pub fn get_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let cursor = activation.context.mouse_data.forced_cursor;
    Ok(cursor
        .map(|cursor| cursor.as_avm2_str(activation))
        .unwrap_or_else(|| istr!("auto"))
        .into())
}

pub fn set_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_string_non_null(activation, 0, "cursor")?;
    // TODO: Support custom cursors set by `flash.ui.Mouse.registerCursor()`.
    let forced = if &value == b"auto" {
        None
    } else {
        Some(
            MouseCursor::from_avm2_str(&value)
                .ok_or_else(|| make_error_2008(activation, "cursor"))?,
        )
    };
    activation.context.mouse_data.forced_cursor = forced;
    Ok(Value::Undefined)
}

pub fn show<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.ui.set_mouse_visible(true);
    Ok(Value::Undefined)
}
