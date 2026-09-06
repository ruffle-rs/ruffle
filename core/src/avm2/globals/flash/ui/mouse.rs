//! `flash.ui.Mouse` builtin

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Avm2StrRepresentable, Error};
use crate::avm2_stub_method;
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
    if let Some(cursor) = activation.context.mouse_data.current_custom_cursor {
        return Ok(cursor.into());
    }

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

    let forced = if &value == b"auto" {
        activation.context.mouse_data.current_custom_cursor = None;
        None
    } else {
        if activation
            .context
            .mouse_data
            .custom_cursors
            .contains(&value)
        {
            avm2_stub_method!(activation, "flash.ui.Mouse", "cursor", "with custom cursor");
            activation.context.mouse_data.current_custom_cursor = Some(value);
            None
        } else {
            Some(
                MouseCursor::from_avm2_str(&value)
                    .ok_or_else(|| make_error_2008(activation, "cursor"))?,
            )
        }
    };

    if forced.is_some() {
        // Set the custom cursor to None now that we know that the cursor
        // is one of the enum values. This needs to be done here
        // so that we don't do this before potentially throwing an error.
        activation.context.mouse_data.current_custom_cursor = None;
    }

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

pub fn register_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.ui.Mouse", "registerCursor");
    let name = args.get_string_non_null(activation, 0, "name")?;
    activation.context.mouse_data.custom_cursors.insert(name);
    Ok(Value::Undefined)
}

pub fn unregister_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.ui.Mouse", "unregisterCursor");
    let name = args.get_string_non_null(activation, 0, "name")?;
    activation.context.mouse_data.custom_cursors.remove(&name);
    if activation.context.mouse_data.current_custom_cursor == Some(name) {
        activation.context.mouse_data.current_custom_cursor = None;
    }
    Ok(Value::Undefined)
}
