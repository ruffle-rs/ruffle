//! AVM1 LoadVars object
//! TODO: bytesLoaded, bytesTotal, contentType, addRequestHeader

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use gc_arena::MutationContext;
use std::borrow::Cow;

/// Implements `LoadVars`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op constructor
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    use Attribute::*;

    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "load",
        load,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "send",
        send,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "sendAndLoad",
        send_and_load,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "decode",
        decode,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "getBytesLoaded",
        get_bytes_loaded,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "getBytesTotal",
        get_bytes_total,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.define_value(
        gc_context,
        "contentType",
        "application/x-www-form-url-encoded".into(),
        DontDelete | DontEnum | ReadOnly,
    );

    object.force_set_function(
        "onLoad",
        on_load,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "onData",
        on_data,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.force_set_function(
        "addRequestHeader",
        add_request_header,
        gc_context,
        DontDelete | DontEnum | ReadOnly,
        Some(fn_proto),
    );

    object.into()
}

fn add_request_header<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("LoadVars.addRequestHeader: Unimplemented");
    Ok(Value::Undefined)
}

fn decode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Spec says added in SWF 7, but not version gated.
    // Decode the query string into properties on this object.
    if let Some(data) = args.get(0) {
        let data = data.coerce_to_string(activation, context)?;
        for (k, v) in url::form_urlencoded::parse(data.as_bytes()) {
            this.set(
                &k,
                crate::avm1::AvmString::new(context.gc_context, v.into_owned()).into(),
                activation,
                context,
            )?;
        }
    }

    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get("_bytesLoaded", activation, context)
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get("_bytesTotal", activation, context)
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url = match args.get(0) {
        Some(val) => val.coerce_to_string(activation, context)?,
        None => return Ok(false.into()),
    };

    spawn_load_var_fetch(activation, context, this, &url, None)?;

    Ok(true.into())
}

fn on_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Default implementation forwards to decode and onLoad.
    let success = match args.get(0) {
        None | Some(Value::Undefined) | Some(Value::Null) => false,
        Some(val) => {
            this.call_method(&"decode", &[val.clone()], activation, context)?;
            this.set("loaded", true.into(), activation, context)?;
            true
        }
    };

    this.call_method(&"onLoad", &[success.into()], activation, context)?;

    Ok(Value::Undefined)
}

fn on_load<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Default implementation: no-op?
    Ok(Value::Undefined)
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // `send` navigates the browser to a URL with the given query parameter.
    let url = match args.get(0) {
        Some(url) => url.coerce_to_string(activation, context)?,
        None => return Ok(false.into()),
    };

    let window = match args.get(1) {
        Some(v) => Some(v.coerce_to_string(activation, context)?),
        None => None,
    };

    let method_name = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation, context)?;
    let method = NavigationMethod::from_method_str(&method_name).unwrap_or(NavigationMethod::POST);

    use std::collections::HashMap;

    let mut form_values = HashMap::new();
    let keys = this.get_keys(activation);

    for k in keys {
        let v = this.get(&k, activation, context);

        form_values.insert(
            k,
            v.ok()
                .unwrap_or_else(|| Value::Undefined)
                .coerce_to_string(activation, context)
                .unwrap_or_else(|_| "undefined".into())
                .to_string(),
        );
    }

    if let Some(window) = window {
        context.navigator.navigate_to_url(
            url.to_string(),
            Some(window.to_string()),
            Some((method, form_values)),
        );
    }

    Ok(true.into())
}

fn send_and_load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);
    let url = url_val.coerce_to_string(activation, context)?;
    let target = match args.get(1) {
        Some(&Value::Object(o)) => o,
        _ => return Ok(false.into()),
    };

    let method_name = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation, context)?;
    let method = NavigationMethod::from_method_str(&method_name).unwrap_or(NavigationMethod::POST);

    spawn_load_var_fetch(activation, context, target, &url, Some((this, method)))?;

    Ok(true.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    use std::collections::HashMap;

    let mut form_values = HashMap::new();
    let keys = this.get_keys(activation);

    for k in keys {
        let v = this.get(&k, activation, context);

        //TODO: What happens if an error occurs inside a virtual property?
        form_values.insert(
            k,
            v.ok()
                .unwrap_or_else(|| Value::Undefined)
                .coerce_to_string(activation, context)
                .unwrap_or_else(|_| "undefined".into())
                .to_string(),
        );
    }

    let query_string = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form_values.iter())
        .finish();

    Ok(crate::avm1::AvmString::new(context.gc_context, query_string).into())
}

fn spawn_load_var_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    loader_object: Object<'gc>,
    url: &AvmString,
    send_object: Option<(Object<'gc>, NavigationMethod)>,
) -> Result<Value<'gc>, Error<'gc>> {
    let (url, request_options) = if let Some((send_object, method)) = send_object {
        // Send properties from `send_object`.
        activation.object_into_request_options(
            context,
            send_object,
            Cow::Borrowed(&url),
            Some(method),
        )
    } else {
        // Not sending any parameters.
        (Cow::Borrowed(url.as_str()), RequestOptions::get())
    };

    let fetch = context.navigator.fetch(&url, request_options);
    let process = context.load_manager.load_form_into_load_vars(
        context.player.clone().unwrap(),
        loader_object,
        fetch,
    );

    // Create hidden properties on object.
    if !loader_object.has_property(activation, context, "_bytesLoaded") {
        loader_object.define_value(
            context.gc_context,
            "_bytesLoaded",
            0.into(),
            Attribute::DontDelete | Attribute::DontEnum,
        );
    } else {
        loader_object.set("_bytesLoaded", 0.into(), activation, context)?;
    }

    if !loader_object.has_property(activation, context, "loaded") {
        loader_object.define_value(
            context.gc_context,
            "loaded",
            false.into(),
            Attribute::DontDelete | Attribute::DontEnum,
        );
    } else {
        loader_object.set("loaded", false.into(), activation, context)?;
    }

    context.navigator.spawn_future(process);

    Ok(true.into())
}
