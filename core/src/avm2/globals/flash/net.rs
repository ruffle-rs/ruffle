//! `flash.net` namespace

use crate::avm2::error::{make_error_2007, reference_error};
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
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

fn parse_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    url: &String,
    data: &Value<'gc>,
) -> Result<(String, IndexMap<String, String>), Error<'gc>> {
    let mut url = url.to_string();
    let mut vars = IndexMap::new();
    let urlvariables = activation
        .avm2()
        .classes()
        .urlvariables
        .inner_class_definition();

    if data.is_of_type(activation, urlvariables) {
        let obj = data
            .as_object()
            .expect("URLVariables object should be Value::Object");
        vars = object_to_index_map(activation, &obj).unwrap_or_default();
    } else if *data != Value::Null {
        let str_data = data.coerce_to_string(activation)?.to_string();
        if !url.contains('?') {
            url.push('?');
        }
        url.push_str(&str_data);
    }

    Ok((url, vars))
}

/// Implements `flash.net.navigateToURL`
pub fn navigate_to_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let request = args.get_object(activation, 0, "request")?;

    let target = args.get_string(activation, 1)?;

    match request.get_public_property("url", activation)? {
        Value::Null => Err(make_error_2007(activation, "url")),
        url => {
            let url = url.coerce_to_string(activation)?.to_string();
            let method = request
                .get_public_property("method", activation)?
                .coerce_to_string(activation)?;
            let method = NavigationMethod::from_method_str(&method).unwrap();
            let data: Value<'gc> = request.get_public_property("data", activation)?;
            let (url, vars) = parse_data(activation, &url, &data)?;
            activation.context.navigator.navigate_to_url(
                &url,
                &target.to_utf8_lossy(),
                Some((method, vars)),
            );
            Ok(Value::Undefined)
        }
    }
}

pub fn register_class_alias<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string_non_null(activation, 0, "aliasName")?;
    let class_object = args
        .get_object(activation, 1, "classObject")?
        .as_class_object()
        .unwrap();

    activation.avm2().register_class_alias(name, class_object);
    Ok(Value::Undefined)
}

pub fn get_class_by_alias<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string_non_null(activation, 0, "aliasName")?;

    if let Some(class_object) = activation.avm2().get_class_by_alias(name) {
        Ok(class_object.into())
    } else {
        // can't create error 1014 normally,
        // as this is one place where it's a ReferenceError for some reason
        let error = reference_error(
            activation,
            &format!("Error #1014: Class {} could not be found.", name),
            1014,
        )?;
        Err(Error::AvmError(error))
    }
}
