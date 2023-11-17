//! `flash.net.URLLoader` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::loader::request_from_url_request;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use crate::loader::DataFormat;

/// Native function definition for `URLLoader.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let request = args.get_object(activation, 0, "request")?;

    let data_format = this
        .get_public_property("dataFormat", activation)?
        .coerce_to_string(activation)?;

    // FIXME: Flash checks the data format after load finishes, before interpreting the data-
    // we check the data format before initializing the load
    let data_format = if &data_format == b"binary" {
        DataFormat::Binary
    } else if &data_format == b"text" {
        DataFormat::Text
    } else if &data_format == b"variables" {
        DataFormat::Variables
    } else {
        tracing::warn!("Unknown URLLoaderDataFormat: {}", data_format);
        DataFormat::Text
    };

    spawn_fetch(activation, this, request, data_format)
}

fn spawn_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    loader_object: Object<'gc>,
    url_request: Object<'gc>,
    data_format: DataFormat,
) -> Result<Value<'gc>, Error<'gc>> {
    let request = request_from_url_request(activation, url_request)?;

    let future = activation.context.load_manager.load_data_into_url_loader(
        activation.context.player.clone(),
        loader_object,
        request,
        data_format,
    );
    activation.context.navigator.spawn_future(future);
    Ok(Value::Undefined)
}
