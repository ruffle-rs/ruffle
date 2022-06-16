//! `flash.net.URLLoader` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use crate::backend::navigator::Request;
use crate::loader::DataFormat;

/// Native function definition for `URLLoader.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let request = match args.get(0) {
            Some(Value::Object(request)) => request,
            // This should never actually happen
            _ => return Ok(Value::Undefined),
        };

        let data_format = this
            .get_property(&QName::dynamic_name("dataFormat").into(), activation)?
            .coerce_to_string(activation)?;

        let data_format = if &data_format == b"binary" {
            DataFormat::Binary
        } else if &data_format == b"text" {
            DataFormat::Text
        } else if &data_format == b"variables" {
            DataFormat::Variables
        } else {
            return Err(format!("Unknown data format: {}", data_format).into());
        };

        return spawn_fetch(activation, this, request, data_format);
    }
    Ok(Value::Undefined)
}

fn spawn_fetch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    loader_object: Object<'gc>,
    url_request: &Object<'gc>,
    data_format: DataFormat,
) -> Result<Value<'gc>, Error> {
    let url = url_request
        .get_property(&QName::dynamic_name("url").into(), activation)?
        .coerce_to_string(activation)?;

    let future = activation.context.load_manager.load_data_into_url_loader(
        activation.context.player.clone(),
        loader_object,
        // FIXME - set options from the `URLRequest`
        Request::get(url.to_string()),
        data_format,
    );
    activation.context.navigator.spawn_future(future);
    Ok(Value::Undefined)
}
