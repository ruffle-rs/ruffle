use crate::avm1::amf::{deserialize_value, recursive_serialize};
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Attribute, Error, NativeObject, Object, Value};
use crate::avm1_stub;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf0::writer::Amf0Writer;
use flash_lso::types::Lso;
use gc_arena::{Collect, Gc};
use ruffle_macros::istr;
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

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "connect" => method(connect; DONT_ENUM | DONT_DELETE);
    "send" => method(send; DONT_ENUM | DONT_DELETE);
    "flush" => method(flush; DONT_ENUM | DONT_DELETE);
    "close" => method(close; DONT_ENUM | DONT_DELETE);
    "getSize" => method(get_size; DONT_ENUM | DONT_DELETE);
    "setFps" => method(set_fps; DONT_ENUM | DONT_DELETE);
    "clear" => method(clear; DONT_ENUM | DONT_DELETE);

    // TODO Looks like onStatus & onSync are not built-in properties.
    "onStatus" => method(on_status; DONT_ENUM | DONT_DELETE);
    "onSync" => method(on_sync; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "deleteAll" => method(delete_all; DONT_ENUM);
    "getDiskUsage" => method(get_disk_usage; DONT_ENUM);
    "getLocal" => method(get_local);
    "getRemote" => method(get_remote);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

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

/// Deserializes a Lso into an object containing the properties stored
fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    lso: &Lso,
    decoder: &AMF0Decoder,
) -> Result<Object<'gc>, Error<'gc>> {
    let obj = Object::new(
        &activation.context.strings,
        Some(activation.prototypes().object),
    );

    let mut reference_cache = BTreeMap::default();

    for child in &lso.body {
        obj.define_value(
            activation.gc(),
            AvmString::new_utf8(activation.gc(), &child.name),
            deserialize_value(activation, child.value(), decoder, &mut reference_cache),
            Attribute::empty(),
        );
    }

    Ok(obj)
}

fn new_lso<'gc>(activation: &mut Activation<'_, 'gc>, name: &str, data: Object<'gc>) -> Lso {
    let mut w = Amf0Writer::default();
    recursive_serialize(activation, data, &mut w);
    w.commit_lso(
        &name
            .split('/')
            .next_back()
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
    let constructor = activation.prototypes().shared_object_constructor;
    let this = constructor
        .construct(activation, &[])?
        .coerce_to_object_or_bare(activation)?;

    // Set the internal name
    if let NativeObject::SharedObject(shared_object) = this.native() {
        shared_object.borrow_mut().set_name(full_name.clone());
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
        data = Object::new(
            &activation.context.strings,
            Some(activation.prototypes().object),
        )
        .into();
    }

    this.define_value(activation.gc(), istr!("data"), data, Attribute::DONT_DELETE);

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
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;

    for k in &data.get_keys(activation, false) {
        data.delete(activation, *k);
    }

    if let NativeObject::SharedObject(shared_object) = this.native() {
        let name = shared_object.borrow().name();
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
    let name = shared_object.borrow().name();
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;
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
    let name = shared_object.borrow().name();
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;
    let mut lso = new_lso(activation, &name, data);
    // Flash returns 0 for empty LSOs, but the actual number of bytes (including the header) otherwise
    if lso.body.is_empty() {
        Ok(0.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(Value::from_usize_lossy(bytes.len()))
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
        activation.gc(),
        NativeObject::SharedObject(Gc::new(activation.gc(), Default::default())),
    );
    Ok(Value::Undefined)
}
