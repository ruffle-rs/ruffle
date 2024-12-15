//! `flash.system` namespace
#![allow(clippy::module_inception)]

pub mod application_domain;
pub mod capabilities;
pub mod security;
pub mod system;

use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

/// Implements `flash.system.fscommand` method
pub fn fscommand<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let command = args.get_string(activation, 0)?;
    let args = args.get_string(activation, 1)?;

    if !activation
        .context
        .external_interface
        .invoke_fs_command(&command.to_utf8_lossy(), &args.to_utf8_lossy())
    {
        tracing::warn!("Unknown FSCommand: {}", command);
    }

    Ok(Value::Undefined)
}
