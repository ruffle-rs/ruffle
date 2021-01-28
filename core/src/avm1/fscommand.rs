//! FSCommand handling

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm_warn;

/// Parse an FSCommand URL.
pub fn parse(url: &str) -> Option<&str> {
    if url.to_lowercase().starts_with("fscommand:") {
        Some(&url["fscommand:".len()..])
    } else {
        None
    }
}

pub fn handle<'gc>(
    command: &str,
    args: &str,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<(), Error<'gc>> {
    if !activation
        .context
        .external_interface
        .invoke_fs_command(command, args)
    {
        avm_warn!(activation, "Unhandled FSCommand: {}", command);
    }
    Ok(())
}
