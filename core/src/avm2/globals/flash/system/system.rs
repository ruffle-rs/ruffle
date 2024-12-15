//! `flash.system.System` native methods

use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::Error;

/// Implements `flash.system.System.setClipboard` method
pub fn set_clipboard<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // The following restrictions only apply to the plugin.
    // TODO: Check the type of event that triggered the function call.
    #[cfg(target_family = "wasm")]
    if false {
        return Err(Error::AvmError(crate::avm2::error::error(
            activation,
            "Error #2176: Certain actions, such as those that display a pop-up window, may only be invoked upon user interaction, for example by a mouse click or button press.",
            2176,
        )?));
    }

    let new_content = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .to_string();

    activation.context.ui.set_clipboard_content(new_content);

    Ok(Value::Undefined)
}
