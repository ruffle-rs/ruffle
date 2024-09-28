//! AVM1 LoadVars object
//! TODO: bytesLoaded, bytesTotal, contentType, addRequestHeader

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::avm1_stub;
use crate::backend::navigator::{NavigationMethod, Request};
use crate::string::{AvmString, StringContext};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "load" => method(load; DONT_ENUM | DONT_DELETE);
    "send" => method(send; DONT_ENUM | DONT_DELETE);
    "sendAndLoad" => method(send_and_load; DONT_ENUM | DONT_DELETE);
    "decode" => method(decode; DONT_ENUM | DONT_DELETE);
    "getBytesLoaded" => method(get_bytes_loaded; DONT_ENUM | DONT_DELETE);
    "getBytesTotal" => method(get_bytes_total; DONT_ENUM | DONT_DELETE);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
    "contentType" => string("application/x-www-form-urlencoded"; DONT_ENUM | DONT_DELETE);
    "onLoad" => method(on_load; DONT_ENUM | DONT_DELETE);
    "onData" => method(on_data; DONT_ENUM | DONT_DELETE);
    "addRequestHeader" => method(add_request_header; DONT_ENUM | DONT_DELETE);
};

/// Implements `LoadVars`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op constructor
    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

fn add_request_header<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "LoadVars", "addRequestHeader");
    Ok(Value::Undefined)
}

fn decode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Spec says added in SWF 7, but not version gated.
    // Decode the query string into properties on this object.
    if let Some(data) = args.get(0) {
        let data = data.coerce_to_string(activation)?;
        for (k, v) in url::form_urlencoded::parse(data.to_utf8_lossy().as_bytes()) {
            let k = AvmString::new_utf8(activation.context.gc_context, k);
            let v = AvmString::new_utf8(activation.context.gc_context, v);
            this.set(k, v.into(), activation)?;
        }
    }

    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get("_bytesLoaded", activation)
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get("_bytesTotal", activation)
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url = match args.get(0) {
        Some(val) => val.coerce_to_string(activation)?,
        None => return Ok(false.into()),
    };

    spawn_load_var_fetch(activation, this, url, None)?;

    Ok(true.into())
}

fn on_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Default implementation forwards to decode and onLoad.
    let success = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => false,
        val => {
            this.call_method(
                "decode".into(),
                &[*val],
                activation,
                ExecutionReason::FunctionCall,
            )?;
            this.set("loaded", true.into(), activation)?;
            true
        }
    };

    this.call_method(
        "onLoad".into(),
        &[success.into()],
        activation,
        ExecutionReason::FunctionCall,
    )?;

    Ok(Value::Undefined)
}

fn on_load<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Default implementation: no-op?
    Ok(Value::Undefined)
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // `send` navigates the browser to a URL with the given query parameter.
    let url = match args.get(0) {
        Some(url) => url.coerce_to_string(activation)?,
        None => return Ok(false.into()),
    };

    let window = match args.get(1) {
        Some(window) => window.coerce_to_string(activation)?,
        None => "".into(),
    };

    let method_name = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let method = NavigationMethod::from_method_str(&method_name).unwrap_or(NavigationMethod::Post);

    use indexmap::IndexMap;

    let mut form_values = IndexMap::new();
    let keys = this.get_keys(activation, false);

    for k in keys {
        let v = this.get(k, activation);

        form_values.insert(
            k.to_string(),
            v.ok()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)
                .unwrap_or_else(|_| "undefined".into())
                .to_string(),
        );
    }

    activation.context.navigator.navigate_to_url(
        &url.to_utf8_lossy(),
        &window.to_utf8_lossy(),
        Some((method, form_values)),
    );

    Ok(true.into())
}

fn send_and_load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);
    let url = url_val.coerce_to_string(activation)?;
    let target = match args.get(1) {
        Some(&Value::Object(o)) => o,
        _ => return Ok(false.into()),
    };

    let method_name = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let method = NavigationMethod::from_method_str(&method_name).unwrap_or(NavigationMethod::Post);

    spawn_load_var_fetch(activation, target, url, Some((this, method)))?;

    Ok(true.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    use indexmap::IndexMap;

    let mut form_values = IndexMap::new();
    let keys = this.get_keys(activation, false);

    for k in keys {
        let v = this.get(k, activation);

        //TODO: What happens if an error occurs inside a virtual property?
        form_values.insert(
            k.to_string(),
            v.ok()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)
                .unwrap_or_else(|_| "undefined".into())
                .to_string(),
        );
    }

    let query_string = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form_values.iter())
        .finish();

    Ok(AvmString::new_utf8(activation.context.gc_context, query_string).into())
}

fn spawn_load_var_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    loader_object: Object<'gc>,
    url: AvmString<'gc>,
    send_object: Option<(Object<'gc>, NavigationMethod)>,
) -> Result<Value<'gc>, Error<'gc>> {
    let request = if let Some((send_object, method)) = send_object {
        // Send properties from `send_object`.
        activation.object_into_request(send_object, url, Some(method))
    } else {
        // Not sending any parameters.
        Request::get(url.to_utf8_lossy().into_owned())
    };

    let future = activation.context.load_manager.load_form_into_load_vars(
        activation.context.player.clone(),
        loader_object,
        request,
    );
    activation.context.navigator.spawn_future(future);

    // Create hidden properties on object.
    if !loader_object.has_property(activation, "_bytesLoaded".into()) {
        loader_object.define_value(
            activation.context.gc_context,
            "_bytesLoaded",
            0.into(),
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        loader_object.set("_bytesLoaded", 0.into(), activation)?;
    }

    if !loader_object.has_property(activation, "_bytesTotal".into()) {
        loader_object.define_value(
            activation.context.gc_context,
            "_bytesTotal",
            Value::Undefined,
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        loader_object.set("_bytesTotal", Value::Undefined, activation)?;
    }

    if !loader_object.has_property(activation, "loaded".into()) {
        loader_object.define_value(
            activation.context.gc_context,
            "loaded",
            false.into(),
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        loader_object.set("loaded", false.into(), activation)?;
    }

    Ok(true.into())
}
