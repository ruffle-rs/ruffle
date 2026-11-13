//! FSCommand handling

use crate::avm_warn;
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::string::WStr;

/// Parse an FSCommand URL.
pub fn parse(url: &WStr) -> Option<&WStr> {
    url.strip_prefix_ignore_case(WStr::from_units(b"fscommand:"))
}

pub fn handle<'gc>(
    command: &WStr,
    args: &WStr,
    activation: &mut Activation<'_, 'gc>,
) -> Result<(), Error<'gc>> {
    let command = command.to_utf8_lossy();
    let args = args.to_utf8_lossy();

    if !activation
        .context
        .external_interface
        .invoke_fs_command(&command, &args)
    {
        avm_warn!(activation, "Unhandled FSCommand: {}", command);
    }
    Ok(())
}
