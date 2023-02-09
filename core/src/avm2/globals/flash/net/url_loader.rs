//! `flash.net.URLLoader` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;
use crate::backend::navigator::{NavigationMethod, Request};
use crate::loader::DataFormat;

/// Native function definition for `URLLoader.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let request = match args.get(0) {
            Some(Value::Object(request)) => request,
            // This should never actually happen
            _ => return Ok(Value::Undefined),
        };

        let data_format = this
            .get_property(&Multiname::public("dataFormat"), activation)?
            .coerce_to_string(activation)?;

        let data_format = if &data_format == b"binary" {
            DataFormat::Binary
        } else if &data_format == b"text" {
            DataFormat::Text
        } else if &data_format == b"variables" {
            DataFormat::Variables
        } else {
            return Err(format!("Unknown data format: {data_format}").into());
        };

        return spawn_fetch(activation, this, request, data_format);
    }
    Ok(Value::Undefined)
}

fn spawn_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    loader_object: Object<'gc>,
    url_request: &Object<'gc>,
    data_format: DataFormat,
) -> Result<Value<'gc>, Error<'gc>> {
    let url = url_request
        .get_property(&Multiname::public("url"), activation)?
        .coerce_to_string(activation)?;

    let method_str = url_request
        .get_property(&Multiname::public("method"), activation)?
        .coerce_to_string(activation)?;

    let method = NavigationMethod::from_method_str(&method_str).unwrap_or_else(|| {
        tracing::error!("Unknown HTTP method type {:?}", method_str);
        NavigationMethod::Get
    });

    let content_type = url_request
        .get_property(&Multiname::public("contentType"), activation)?
        .coerce_to_string(activation)?;

    let data = url_request.get_property(&Multiname::public("data"), activation)?;

    let data = if let Value::Null = data {
        None
    } else {
        Some(data.coerce_to_object(activation)?)
    };

    // FIXME - set options from the `URLRequest`
    let mut request = Request::request(method, url.to_string(), None);

    if let Some(data) = data {
        if data.is_of_type(activation.avm2().classes().urlvariables, activation) {
            if &*content_type == b"application/x-www-form-urlencoded" {
                let data = data
                    .call_property(&Multiname::public("toString"), &[], activation)?
                    .coerce_to_string(activation)?
                    .to_string()
                    .into_bytes();
                if &*method_str == b"GET" {
                    avm2_stub_method!(
                        activation,
                        "flash.net.URLLoader",
                        "load",
                        "with GET method and URLVariables data"
                    );
                }
                request.set_body((data, "application/x-www-form-urlencoded".to_string()));
            } else {
                avm2_stub_method!(
                    activation,
                    "flash.net.URLLoader",
                    "load",
                    "with URLVariables data and content type other than application/x-www-form-urlencoded"
                );
            }
        } else {
            avm2_stub_method!(
                activation,
                "flash.net.URLLoader",
                "load",
                "with non-URLVariables data"
            );
        }
    }

    let future = activation.context.load_manager.load_data_into_url_loader(
        activation.context.player.clone(),
        loader_object,
        request,
        data_format,
    );
    activation.context.navigator.spawn_future(future);
    Ok(Value::Undefined)
}
