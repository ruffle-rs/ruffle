//! `flash.net` namespace

use crate::avm2::error::type_error;
use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};

pub mod local_connection;
pub mod file_reference;
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

    match request.get_public_property("url", activation)? {
        Value::Null => Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter url must be non-null.",
            2007,
        )?)),
        url => {
            let url = url.coerce_to_string(activation)?;
            activation.context.navigator.navigate_to_url(
                &url.to_utf8_lossy(),
                &target.to_utf8_lossy(),
                None,
            );
            Ok(Value::Undefined)
        }
    }
}
