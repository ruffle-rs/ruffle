//! FSCommand handling

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm_warn;

/// Parse an FSCommand URL.
pub fn parse(url: &str) -> Option<&str> {
    log::info!("Checking {}", url);
    if url.to_lowercase().starts_with("fscommand:") {
        Some(&url["fscommand:".len()..])
    } else {
        None
    }
}

/// TODO: FSCommand URL handling
pub fn handle<'gc>(
    fscommand: &str,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "Unhandled FSCommand: {}", fscommand);

    //This should be an error.
    Ok(())
}
