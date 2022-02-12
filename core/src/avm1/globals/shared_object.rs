use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::shared_object::SharedObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use flash_lso::types::Value as AmfValue;
use flash_lso::types::{AMFVersion, Element, Lso};
use gc_arena::MutationContext;
use json::JsonValue;
use std::borrow::Cow;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "clear" => method(clear; DONT_ENUM | DONT_DELETE);
    "close" => method(close; DONT_ENUM | DONT_DELETE);
    "connect" => method(connect; DONT_ENUM | DONT_DELETE);
    "flush" => method(flush; DONT_ENUM | DONT_DELETE);
    "getSize" => method(get_size; DONT_ENUM | DONT_DELETE);
    "send" => method(send; DONT_ENUM | DONT_DELETE);
    "setFps" => method(set_fps; DONT_ENUM | DONT_DELETE);
    "onStatus" => method(on_status; DONT_ENUM | DONT_DELETE);
    "onSync" => method(on_sync; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "deleteAll" => method(delete_all; DONT_ENUM);
    "getDiskUsage" => method(get_disk_usage; DONT_ENUM);
    "getLocal" => method(get_local);
    "getRemote" => method(get_remote);
    "getMaxSize" => method(get_max_size);
    "addListener" => method(add_listener);
    "removeListener" => method(remove_listener);
};

pub fn delete_all<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.deleteAll() not implemented");
    Ok(Value::Undefined)
}

pub fn get_disk_usage<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getDiskUsage() not implemented");
    Ok(Value::Undefined)
}

/// Serialize a Value to an AmfValue
fn serialize_value<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    elem: Value<'gc>,
) -> Option<AmfValue> {
    match elem {
        Value::Undefined => Some(AmfValue::Undefined),
        Value::Null => Some(AmfValue::Null),
        Value::Bool(b) => Some(AmfValue::Bool(b)),
        Value::Number(f) => Some(AmfValue::Number(f)),
        Value::String(s) => Some(AmfValue::String(s.to_string())),
        Value::Object(o) => {
            // TODO: Find a more general rule for which object types should be skipped,
            // and which turn into undefined.
            if o.as_executable().is_some() {
                None
            } else if o.as_display_object().is_some() {
                Some(AmfValue::Undefined)
            } else if o.as_array_object().is_some() {
                let mut values = Vec::new();
                recursive_serialize(activation, o, &mut values);

                // TODO: What happens if an exception is thrown here?
                let length = o.length(activation).unwrap();
                Some(AmfValue::ECMAArray(vec![], values, length as u32))
            } else if let Some(xml_node) = o.as_xml_node() {
                Some(AmfValue::XML(
                    xml_node.into_string().to_utf8_lossy().into_owned(),
                    true,
                ))
            } else if let Some(date) = o.as_date_object() {
                date.date_time()
                    .map(|date_time| AmfValue::Date(date_time.timestamp_millis() as f64, None))
            } else {
                let mut object_body = Vec::new();
                recursive_serialize(activation, o, &mut object_body);
                Some(AmfValue::Object(object_body, None))
            }
        }
    }
}

/// Serialize an Object and any children to a JSON object
fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    obj: Object<'gc>,
    elements: &mut Vec<Element>,
) {
    // Reversed to match flash player ordering
    for element_name in obj.get_keys(activation).into_iter().rev() {
        if let Ok(elem) = obj.get(element_name, activation) {
            if let Some(v) = serialize_value(activation, elem) {
                elements.push(Element::new(element_name.to_utf8_lossy(), v));
            }
        }
    }
}

/// Deserialize a AmfValue to a Value
fn deserialize_value<'gc>(activation: &mut Activation<'_, 'gc, '_>, val: &AmfValue) -> Value<'gc> {
    match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.context.gc_context, s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ECMAArray(_, associative, len) => {
            let array_constructor = activation.context.avm1.prototypes.array_constructor;
            if let Ok(Value::Object(obj)) =
                array_constructor.construct(activation, &[(*len).into()])
            {
                for entry in associative {
                    let value = deserialize_value(activation, entry.value());

                    if let Ok(i) = entry.name().parse::<i32>() {
                        obj.set_element(activation, i, value).unwrap();
                    } else {
                        obj.define_value(
                            activation.context.gc_context,
                            AvmString::new_utf8(activation.context.gc_context, &entry.name),
                            value,
                            Attribute::empty(),
                        );
                    }
                }

                obj.into()
            } else {
                Value::Undefined
            }
        }
        AmfValue::Object(elements, _) => {
            // Deserialize Object
            let obj = ScriptObject::object(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes.object),
            );
            for entry in elements {
                let value = deserialize_value(activation, entry.value());
                let name = AvmString::new_utf8(activation.context.gc_context, &entry.name);
                obj.define_value(
                    activation.context.gc_context,
                    name,
                    value,
                    Attribute::empty(),
                );
            }
            obj.into()
        }
        AmfValue::Date(time, _) => {
            let date_proto = activation.context.avm1.prototypes.date_constructor;

            if let Ok(Value::Object(obj)) = date_proto.construct(activation, &[(*time).into()]) {
                Value::Object(obj)
            } else {
                Value::Undefined
            }
        }
        AmfValue::XML(content, _) => {
            let xml_proto = activation.context.avm1.prototypes.xml_constructor;

            if let Ok(Value::Object(obj)) = xml_proto.construct(
                activation,
                &[Value::String(AvmString::new_utf8(
                    activation.context.gc_context,
                    content,
                ))],
            ) {
                Value::Object(obj)
            } else {
                Value::Undefined
            }
        }

        _ => Value::Undefined,
    }
}

/// Deserializes a Lso into an object containing the properties stored
fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    lso: &Lso,
) -> Result<Object<'gc>, Error<'gc>> {
    let obj = ScriptObject::object(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes.object),
    );

    for child in &lso.body {
        obj.define_value(
            activation.context.gc_context,
            AvmString::new_utf8(activation.context.gc_context, &child.name),
            deserialize_value(activation, child.value()),
            Attribute::empty(),
        );
    }

    Ok(obj.into())
}

/// Deserialize a Json shared object element into a Value
fn recursive_deserialize_json<'gc>(
    json_value: &JsonValue,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    match json_value {
        JsonValue::Null => Value::Null,
        JsonValue::Short(s) => Value::String(AvmString::new_utf8(
            activation.context.gc_context,
            s.to_string(),
        )),
        JsonValue::String(s) => {
            Value::String(AvmString::new_utf8(activation.context.gc_context, s))
        }
        JsonValue::Number(f) => Value::Number((*f).into()),
        JsonValue::Boolean(b) => (*b).into(),
        JsonValue::Object(o) => {
            if o.get("__proto__").and_then(JsonValue::as_str) == Some("Array") {
                deserialize_array_json(o, activation)
            } else {
                deserialize_object_json(o, activation)
            }
        }
        JsonValue::Array(_) => Value::Undefined,
    }
}

/// Deserialize an Object and any children from a JSON object
fn deserialize_object_json<'gc>(
    json_obj: &json::object::Object,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    // Deserialize Object
    let obj = ScriptObject::object(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes.object),
    );
    for (name, value) in json_obj.iter() {
        obj.define_value(
            activation.context.gc_context,
            AvmString::new_utf8(activation.context.gc_context, name),
            recursive_deserialize_json(value, activation),
            Attribute::empty(),
        );
    }
    obj.into()
}

/// Deserialize an Array and any children from a JSON object
fn deserialize_array_json<'gc>(
    json_obj: &json::object::Object,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    let array_constructor = activation.context.avm1.prototypes.array_constructor;
    let len = json_obj
        .get("length")
        .and_then(JsonValue::as_i32)
        .unwrap_or_default();
    if let Ok(Value::Object(obj)) = array_constructor.construct(activation, &[len.into()]) {
        for (name, value) in json_obj.iter() {
            let value = recursive_deserialize_json(value, activation);
            if let Ok(i) = name.parse::<i32>() {
                obj.set_element(activation, i, value).unwrap();
            } else {
                // Ignore length and proto meta-properties
                if name == "length" || name == "__proto__" {
                    continue;
                }

                obj.define_value(
                    activation.context.gc_context,
                    AvmString::new_utf8(activation.context.gc_context, name),
                    value,
                    Attribute::empty(),
                );
            }
        }

        obj.into()
    } else {
        Value::Undefined
    }
}

pub fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: It appears that Flash does some kind of escaping here:
    // the name "foo\uD800" correspond to a file named "fooE#FB#FB#D.sol".

    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let name = name.to_utf8_lossy();

    const INVALID_CHARS: &str = "~%&\\;:\"',<>?# ";
    if name.contains(|c| INVALID_CHARS.contains(c)) {
        log::error!("SharedObject::get_local: Invalid character in name");
        return Ok(Value::Null);
    }

    let movie = if let Some(movie) = activation.base_clip().movie() {
        movie
    } else {
        log::error!("SharedObject::get_local: Movie was None");
        return Ok(Value::Null);
    };

    let mut movie_url = if let Some(url) = movie.url() {
        if let Ok(url) = url::Url::parse(url) {
            url
        } else {
            log::error!("SharedObject::get_local: Unable to parse movie URL");
            return Ok(Value::Null);
        }
    } else {
        // No URL (loading local data). Use a dummy URL to allow SharedObjects to work.
        url::Url::parse("file://localhost").unwrap()
    };
    movie_url.set_query(None);
    movie_url.set_fragment(None);

    let secure = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .as_bool(activation.swf_version());

    // Secure parameter disallows using the shared object from non-HTTPS.
    if secure && movie_url.scheme() != "https" {
        log::warn!(
            "SharedObject.get_local: Tried to load a secure shared object from non-HTTPS origin"
        );
        return Ok(Value::Null);
    }

    // Shared objects are sandboxed per-domain.
    // By default, they are keyed based on the SWF URL, but the `localHost` parameter can modify this path.
    let mut movie_path = movie_url.path();
    // Remove leading/trailing slashes.
    movie_path = movie_path.strip_prefix('/').unwrap_or(movie_path);
    movie_path = movie_path.strip_suffix('/').unwrap_or(movie_path);

    let movie_host = if movie_url.scheme() == "file" {
        // Remove drive letter on Windows (TODO: move this logic into DiskStorageBackend?)
        if let [_, b':', b'/', ..] = movie_path.as_bytes() {
            movie_path = &movie_path[3..];
        }
        "localhost"
    } else {
        movie_url.host_str().unwrap_or_default()
    };

    let local_path = if let Some(Value::String(local_path)) = args.get(1) {
        // Empty local path always fails.
        if local_path.is_empty() {
            return Ok(Value::Null);
        }

        // Remove leading/trailing slashes.
        let mut local_path = local_path.to_utf8_lossy();
        if local_path.ends_with('/') {
            match &mut local_path {
                Cow::Owned(p) => {
                    p.pop();
                }
                Cow::Borrowed(p) => *p = &p[..p.len() - 1],
            }
        }
        if local_path.starts_with('/') {
            match &mut local_path {
                Cow::Owned(p) => {
                    p.remove(0);
                }
                Cow::Borrowed(p) => *p = &p[1..],
            }
        }

        // Verify that local_path is a prefix of the SWF path.
        if movie_path.starts_with(local_path.as_ref())
            && (local_path.is_empty()
                || movie_path.len() == local_path.len()
                || movie_path[local_path.len()..].starts_with('/'))
        {
            local_path
        } else {
            log::warn!("SharedObject.get_local: localPath parameter does not match SWF path");
            return Ok(Value::Null);
        }
    } else {
        Cow::Borrowed(movie_path)
    };

    // Final SO path: foo.com/folder/game.swf/SOName
    // SOName may be a path containing slashes. In this case, prefix with # to mimic Flash Player behavior.
    let prefix = if name.contains('/') { "#" } else { "" };
    let full_name = format!("{}/{}/{}{}", movie_host, local_path, prefix, name);

    // Avoid any paths with `..` to prevent SWFs from crawling the file system on desktop.
    // Flash will generally fail to save shared objects with a path component starting with `.`,
    // so let's disallow them altogether.
    if full_name.split('/').any(|s| s.starts_with('.')) {
        log::error!("SharedObject.get_local: Invalid path with .. segments");
        return Ok(Value::Null);
    }

    // Check if this is referencing an existing shared object
    if let Some(so) = activation.context.shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    // Data property only should exist when created with getLocal/Remote
    let constructor = activation.context.avm1.prototypes.shared_object_constructor;
    let this = constructor
        .construct(activation, &[])?
        .coerce_to_object(activation);

    // Set the internal name
    let obj_so = this.as_shared_object().unwrap();
    obj_so.set_name(activation.context.gc_context, full_name.clone());

    let mut data = Value::Undefined;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name) {
        // Attempt to load it as an Lso
        if let Ok(lso) = flash_lso::read::Reader::default().parse(&saved) {
            data = deserialize_lso(activation, &lso)?.into();
        } else {
            // Attempt to load legacy Json
            if let Ok(saved_string) = String::from_utf8(saved) {
                if let Ok(json_data) = json::parse(&saved_string) {
                    data = recursive_deserialize_json(&json_data, activation);
                }
            }
        }
    }

    if data == Value::Undefined {
        // No data; create a fresh data object.
        data = ScriptObject::object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.object),
        )
        .into();
    }

    this.define_value(
        activation.context.gc_context,
        "data",
        data,
        Attribute::DONT_DELETE,
    );

    activation.context.shared_objects.insert(full_name, this);

    Ok(this.into())
}

pub fn get_remote<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getRemote() not implemented");
    Ok(Value::Undefined)
}

pub fn get_max_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getMaxSize() not implemented");
    Ok(Value::Undefined)
}

pub fn add_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.addListener() not implemented");
    Ok(Value::Undefined)
}

pub fn remove_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.removeListener() not implemented");
    Ok(Value::Undefined)
}

pub fn create_shared_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    shared_object_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        Some(fn_proto),
        shared_object_proto,
    );
    let object = shared_obj.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    shared_obj
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this.get("data", activation)?.coerce_to_object(activation);

    for k in &data.get_keys(activation) {
        data.delete(activation, *k);
    }

    let so = this.as_shared_object().unwrap();
    let name = so.get_name();

    activation.context.storage.remove_key(&name);

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.close() not implemented");
    Ok(Value::Undefined)
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.connect() not implemented");
    Ok(Value::Undefined)
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this.get("data", activation)?.coerce_to_object(activation);

    let this_obj = this.as_shared_object().unwrap();
    let name = this_obj.get_name();

    let mut elements = Vec::new();
    recursive_serialize(activation, data, &mut elements);
    let mut lso = Lso::new(
        elements,
        &name
            .split('/')
            .last()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        AMFVersion::AMF0,
    );

    let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();

    Ok(activation.context.storage.put(&name, &bytes).into())
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getSize() not implemented");
    Ok(Value::Undefined)
}

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.send() not implemented");
    Ok(Value::Undefined)
}

pub fn set_fps<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.setFps() not implemented");
    Ok(Value::Undefined)
}

pub fn on_status<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.onStatus() not implemented");
    Ok(Value::Undefined)
}

pub fn on_sync<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.onSync() not implemented");
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj = SharedObject::empty_shared_obj(gc_context, Some(proto));
    let object = shared_obj.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    shared_obj.into()
}

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}
