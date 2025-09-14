//! FSCommand handling

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm_warn;
use ruffle_wstr::WStr;

/// Parse an FSCommand URL.
pub fn parse(url: &WStr) -> Option<&WStr> {
    let prefix = WStr::from_units(b"fscommand:");
    if url.len() < prefix.len() {
        return None;
    }

    let (head, tail) = url.split_at(prefix.len());
    if head.eq_ignore_case(prefix) {
        Some(tail)
    } else {
        None
    }
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
