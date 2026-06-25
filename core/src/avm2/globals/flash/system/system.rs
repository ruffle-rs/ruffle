//! `flash.system.System` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;

/// Implements `flash.system.System.gc` method.
///
/// In Flash Player this is a debug-mode hint; release builds ignore it.
/// In Ruffle, the call sets the `Avm2::force_full_gc` flag; `Player::update`
/// reads the flag and runs a full `gc-arena finish_cycle` after the current
/// update completes. This promotes `System.gc()` to a first-class trigger
/// for full collection, usable both by diagnostic tooling and by application
/// code that wants to drain weak references and reclaim slot space at
/// well-defined points (typically after teardown of a large UI subtree).
pub fn gc<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.avm2.force_full_gc.set(true);
    Ok(Value::Undefined)
}

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
        return Err(crate::avm2::error::make_error_2176(activation));
    }

    let new_content = args.get_string_non_null(activation, 0, "text")?;
    activation
        .context
        .ui
        .set_clipboard_content(new_content.to_string());

    Ok(Value::Undefined)
}
