//! FSCommand handling

use crate::avm1::{Avm1, ActionContext, Error};

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
pub fn handle(fscommand: &str, _avm: &mut Avm1, _ac: &mut ActionContext) -> Result<(), Error> {
    log::warn!("Unhandled FSCommand: {}", fscommand);

    //This should be an error.
    Ok(())
}