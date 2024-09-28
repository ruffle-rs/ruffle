use crate::avm1::function::FunctionObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{
    Activation, Attribute, Error, Executable, NativeObject, Object, ScriptObject, TObject, Value,
};
use crate::avm1_stub;
use crate::display_object::TDisplayObject;
use crate::string::{AvmString, StringContext};
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf0::writer::{Amf0Writer, CacheKey, ObjWriter};
use flash_lso::types::{Lso, Reference, Value as AmfValue};
use gc_arena::{Collect, GcCell};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Default, Clone, Collect)]
#[collect(require_static)]
pub struct SharedObject {
    /// The local name of this shared object
    name: Option<String>,
    // In future this will also handle remote SharedObjects
}

impl SharedObject {
    fn name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

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
};

fn delete_all<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "deleteAll");
    Ok(Value::Undefined)
}

fn get_disk_usage<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "getDiskUsage");
    Ok(Value::Undefined)
}

pub fn serialize<'gc>(activation: &mut Activation<'_, 'gc>, value: Value<'gc>) -> AmfValue {
    match value {
        Value::Undefined => AmfValue::Undefined,
        Value::Null => AmfValue::Null,
        Value::Bool(bool) => AmfValue::Bool(bool),
        Value::Number(number) => AmfValue::Number(number),
        Value::String(string) => AmfValue::String(string.to_string()),
        Value::Object(object) => {
            let lso = new_lso(activation, "root", object);
            AmfValue::Object(lso.into_iter().collect(), None)
        }
        Value::MovieClip(_) => AmfValue::Undefined,
    }
}

/// Serialize an Object and any children to a JSON object
fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    writer: &mut dyn ObjWriter<'_>,
) {
    // Reversed to match flash player ordering
    for element_name in obj.get_keys(activation, false).into_iter().rev() {
        if let Ok(elem) = obj.get(element_name, activation) {
            let name = element_name.to_utf8_lossy();

            match elem {
                Value::Object(o) => {
                    if o.as_executable().is_some() {
                    } else if o.as_display_object().is_some() {
                        writer.undefined(name.as_ref())
                    } else if o.as_array_object().is_some() {
                        let (aw, token) = writer.array(CacheKey::from_ptr(o.as_ptr()));

                        if let Some(mut aw) = aw {
                            recursive_serialize(activation, o, &mut aw);

                            // TODO: What happens if an exception is thrown here?
                            let length = o
                                .length(activation)
                                .expect("Failed to get length for SharedObject array");

                            aw.commit(name, length as u32);
                        } else {
                            writer.reference(name.as_ref(), token);
                        }
                    } else if let Some(xml_node) = o.as_xml_node() {
                        // TODO: What happens if an exception is thrown here?
                        let string = xml_node
                            .into_string(activation)
                            .expect("Failed to convert xml to string in SharedObject");
                        writer.xml(name.as_ref(), string.to_utf8_lossy().as_ref(), true)
                    } else if let NativeObject::Date(date) = o.native() {
                        writer.date(name.as_ref(), date.get().time(), None)
                    } else {
                        let (ow, token) = writer.object(CacheKey::from_ptr(o.as_ptr()));

                        if let Some(mut ow) = ow {
                            recursive_serialize(activation, o, &mut ow);
                            ow.commit(name);
                        } else {
                            writer.reference(name.as_ref(), token);
                        }
                    }
                }
                Value::Number(f) => writer.number(name.as_ref(), f),
                Value::String(s) => writer.string(name.as_ref(), s.to_utf8_lossy().as_ref()),
                Value::Undefined | Value::MovieClip(_) => writer.undefined(name.as_ref()),
                Value::Null => writer.null(name.as_ref()),
                Value::Bool(b) => writer.bool(name.as_ref(), b),
            }
        }
    }
}

/// Deserialize a AmfValue to a Value
pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    lso: &AMF0Decoder,
    reference_cache: &mut BTreeMap<Reference, Value<'gc>>,
) -> Value<'gc> {
    match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.context.gc_context, s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ECMAArray(_, associative, len) => {
            let array_constructor = activation.context.avm1.prototypes().array_constructor;
            if let Ok(Value::Object(obj)) =
                array_constructor.construct(activation, &[(*len).into()])
            {
                let v: Value<'gc> = obj.into();

                // This should always be valid, but lets be sure
                if let Some(reference) = lso.as_reference(val) {
                    reference_cache.insert(reference, v);
                }

                for entry in associative {
                    let value = deserialize_value(activation, entry.value(), lso, reference_cache);

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

                v
            } else {
                Value::Undefined
            }
        }
        AmfValue::Object(elements, _) => {
            // Deserialize Object
            let obj = ScriptObject::new(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes().object),
            );

            let v: Value<'gc> = obj.into();

            // This should always be valid, but lets be sure
            if let Some(reference) = lso.as_reference(val) {
                reference_cache.insert(reference, v);
            }

            for entry in elements {
                let value = deserialize_value(activation, entry.value(), lso, reference_cache);
                let name = AvmString::new_utf8(activation.context.gc_context, &entry.name);
                obj.define_value(
                    activation.context.gc_context,
                    name,
                    value,
                    Attribute::empty(),
                );
            }

            v
        }
        AmfValue::Date(time, _) => {
            let date_proto = activation.context.avm1.prototypes().date_constructor;

            if let Ok(Value::Object(obj)) = date_proto.construct(activation, &[(*time).into()]) {
                Value::Object(obj)
            } else {
                Value::Undefined
            }
        }
        AmfValue::XML(content, _) => {
            let xml_proto = activation.context.avm1.prototypes().xml_constructor;

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
        AmfValue::Reference(x) => {
            // This should always be a valid reference, but a "bad" file could create an invalid one
            // In that case we will just assume undefined
            let val = reference_cache.get(x).unwrap_or(&Value::Undefined);
            *val
        }
        _ => Value::Undefined,
    }
}

/// Deserializes a Lso into an object containing the properties stored
fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    lso: &Lso,
    decoder: &AMF0Decoder,
) -> Result<Object<'gc>, Error<'gc>> {
    let obj = ScriptObject::new(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes().object),
    );

    let mut reference_cache = BTreeMap::default();

    for child in &lso.body {
        obj.define_value(
            activation.context.gc_context,
            AvmString::new_utf8(activation.context.gc_context, &child.name),
            deserialize_value(activation, child.value(), decoder, &mut reference_cache),
            Attribute::empty(),
        );
    }

    Ok(obj.into())
}

fn new_lso<'gc>(activation: &mut Activation<'_, 'gc>, name: &str, data: Object<'gc>) -> Lso {
    let mut w = Amf0Writer::default();
    recursive_serialize(activation, data, &mut w);
    w.commit_lso(
        &name
            .split('/')
            .last()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
    )
}

fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
        tracing::error!("SharedObject::get_local: Invalid character in name");
        return Ok(Value::Null);
    }

    let movie = activation.base_clip().movie();

    let mut movie_url = if let Ok(url) = url::Url::parse(movie.url()) {
        url
    } else {
        tracing::error!("SharedObject::get_local: Unable to parse movie URL");
        return Ok(Value::Null);
    };
    movie_url.set_query(None);
    movie_url.set_fragment(None);

    let secure = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .as_bool(activation.swf_version());

    // Secure parameter disallows using the shared object from non-HTTPS.
    if secure && movie_url.scheme() != "https" {
        tracing::warn!(
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
            tracing::warn!("SharedObject.get_local: localPath parameter does not match SWF path");
            return Ok(Value::Null);
        }
    } else {
        Cow::Borrowed(movie_path)
    };

    // Final SO path: foo.com/folder/game.swf/SOName
    // SOName may be a path containing slashes. In this case, prefix with # to mimic Flash Player behavior.
    let prefix = if name.contains('/') { "#" } else { "" };
    let full_name = format!("{movie_host}/{local_path}/{prefix}{name}");

    // Avoid any paths with `..` to prevent SWFs from crawling the file system on desktop.
    // Flash will generally fail to save shared objects with a path component starting with `.`,
    // so let's disallow them altogether.
    if full_name.split('/').any(|s| s.starts_with('.')) {
        tracing::error!("SharedObject.get_local: Invalid path with .. segments");
        return Ok(Value::Null);
    }

    // Check if this is referencing an existing shared object
    if let Some(so) = activation.context.avm1_shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    // Data property only should exist when created with getLocal/Remote
    let constructor = activation
        .context
        .avm1
        .prototypes()
        .shared_object_constructor;
    let this = constructor
        .construct(activation, &[])?
        .coerce_to_object(activation);

    // Set the internal name
    if let NativeObject::SharedObject(shared_object) = this.native() {
        shared_object
            .write(activation.context.gc_context)
            .set_name(full_name.clone());
    }

    let mut data = Value::Undefined;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name) {
        let mut reader = flash_lso::read::Reader::default();
        if let Ok(lso) = reader.parse(&saved) {
            data = deserialize_lso(activation, &lso, &reader.amf0_decoder)?.into();
        }
    }

    if data == Value::Undefined {
        // No data; create a fresh data object.
        data = ScriptObject::new(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes().object),
        )
        .into();
    }

    this.define_value(
        activation.context.gc_context,
        "data",
        data,
        Attribute::DONT_DELETE,
    );

    activation
        .context
        .avm1_shared_objects
        .insert(full_name, this);

    Ok(this.into())
}

fn get_remote<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "getRemote");
    Ok(Value::Undefined)
}

fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this.get("data", activation)?.coerce_to_object(activation);

    for k in &data.get_keys(activation, false) {
        data.delete(activation, *k);
    }

    if let NativeObject::SharedObject(shared_object) = this.native() {
        let name = shared_object.read().name();
        activation.context.storage.remove_key(&name);
    }

    Ok(Value::Undefined)
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "close");
    Ok(Value::Undefined)
}

fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "connect");
    Ok(Value::Undefined)
}

pub(crate) fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::SharedObject(shared_object) = this.native() else {
        return Ok(Value::Undefined);
    };
    let name = shared_object.read().name();
    let data = this.get("data", activation)?.coerce_to_object(activation);
    let mut lso = new_lso(activation, &name, data);
    flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
    // Flash does not write empty LSOs to disk
    if lso.body.is_empty() {
        Ok(true.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(activation.context.storage.put(&name, &bytes).into())
    }
}

fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::SharedObject(shared_object) = this.native() else {
        return Ok(Value::Undefined);
    };
    let name = shared_object.read().name();
    let data = this.get("data", activation)?.coerce_to_object(activation);
    let mut lso = new_lso(activation, &name, data);
    // Flash returns 0 for empty LSOs, but the actual number of bytes (including the header) otherwise
    if lso.body.is_empty() {
        Ok(0.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(bytes.len().into())
    }
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "send");
    Ok(Value::Undefined)
}

fn set_fps<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "setFps");
    Ok(Value::Undefined)
}

fn on_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "onStatus");
    Ok(Value::Undefined)
}

fn on_sync<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "onSync");
    Ok(Value::Undefined)
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_native(
        activation.context.gc_context,
        NativeObject::SharedObject(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        )),
    );
    Ok(this.into())
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_object_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, shared_object_proto, fn_proto);
    let constructor = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        shared_object_proto.into(),
    );
    define_properties_on(
        OBJECT_DECLS,
        context,
        constructor.raw_script_object(),
        fn_proto,
    );
    constructor
}
