//! `flash.net` namespace

use crate::avm2::globals::flash::display::loader::request_from_url_request;
use crate::avm2::{Activation, Error, Object, Value};

pub mod file_reference;
pub mod local_connection;
pub mod net_connection;
pub mod net_stream;
pub mod object_encoding;
pub mod responder;
pub mod shared_object;
pub mod socket;
pub mod url_loader;
pub mod xml_socket;

/// Implements `flash.net.navigateToURL`
pub fn navigate_to_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_request = args
        .get(0)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_object(activation)?;

    let target = args
        .get(1)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_string(activation)?;

    let request = request_from_url_request(activation, url_request)?;
    activation
        .context
        .navigator
        .navigate_to_url(request, &target.to_utf8_lossy());
    Ok(Value::Undefined)
}
