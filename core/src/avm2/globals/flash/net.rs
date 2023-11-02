//! `flash.net` namespace

use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};

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
    let request = args
        .get(0)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_object(activation)?;

    let target = args
        .get(1)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_string(activation)?;

    let url = request
        .get_public_property("url", activation)?
        .coerce_to_string(activation)?;

    activation.context.navigator.navigate_to_url(
        &url.to_utf8_lossy(),
        &target.to_utf8_lossy(),
        None,
    );

    Ok(Value::Undefined)
}
