//! `flash.net` namespace

use crate::avm2::error::type_error;
use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};
use crate::backend::navigator::NavigationMethod;
use indexmap::IndexMap;

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

fn object_to_index_map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: &Object<'gc>,
) -> Result<IndexMap<String, String>, Error<'gc>> {
    let mut map = IndexMap::new();
    let mut last_index = obj.get_next_enumerant(0, activation)?;
    while let Some(index) = last_index {
        let name = obj
            .get_enumerant_name(index, activation)?
            .coerce_to_string(activation)?;
        let value = obj
            .get_public_property(name, activation)?
            .coerce_to_string(activation)?
            .to_utf8_lossy()
            .to_string();

        let name = name.to_utf8_lossy().to_string();
        map.insert(name, value);
        last_index = obj.get_next_enumerant(index, activation)?;
    }
    Ok(map)
}

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
            let method = request
                .get_public_property("method", activation)?
                .coerce_to_string(activation)?;
            let method =
                NavigationMethod::from_method_str(&method).unwrap();
            let data = request
                .get_public_property("data", activation)?
                .coerce_to_object(activation)?;
            // If data is byte array this will not work
            let data = object_to_index_map(activation, &data).unwrap_or_default();
            activation.context.navigator.navigate_to_url(
                &url.to_utf8_lossy(),
                &target.to_utf8_lossy(),
                Some((method, data)),
            );
            Ok(Value::Undefined)
        }
    }
}
