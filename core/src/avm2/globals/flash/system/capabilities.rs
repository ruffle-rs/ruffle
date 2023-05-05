//! `flash.display.Capabilities` native methods

use crate::avm2::{Activation, AvmString, Error, Object, Value};

/// Implements `flash.system.Capabilities.version`
pub fn get_version<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Report the correct OS instead of always reporting Linux
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!("LNX {},0,0,0", activation.avm2().player_version),
    )
    .into())
}

/// Implements `flash.system.Capabilities.playerType`
pub fn get_player_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: When should "External" be returned?
    let player_type = if cfg!(target_family = "wasm") {
        "PlugIn"
    } else {
        "StandAlone"
    };

    Ok(AvmString::new_utf8(activation.context.gc_context, player_type).into())
}
