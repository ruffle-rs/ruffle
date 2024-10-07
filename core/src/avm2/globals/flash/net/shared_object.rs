//! `flash.net.SharedObject` builtin/prototype

use crate::avm2::error::error;
use crate::avm2::object::TObject;
pub use crate::avm2::object::{shared_object_allocator, SharedObjectObject};
use crate::avm2::{Activation, Error, Object, Value};
use crate::{avm2_stub_getter, avm2_stub_method, avm2_stub_setter};
use flash_lso::types::{AMFVersion, Lso};
use std::borrow::Cow;

fn new_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &str,
    data: Object<'gc>,
) -> Result<Lso, Error<'gc>> {
    let mut elements = Vec::new();
    crate::avm2::amf::recursive_serialize(
        activation,
        data,
        &mut elements,
        None,
        AMFVersion::AMF3,
        &mut Default::default(),
    )?;
    Ok(Lso::new(
        elements,
        name.split('/')
            .last()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        AMFVersion::AMF3,
    ))
}

pub fn get_local<'gc>(
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

    let mut movie_url = if let Ok(url) = url::Url::parse(activation.context.swf.url()) {
        url
    } else {
        tracing::error!("SharedObject::get_local: Unable to parse movie URL");
        return Ok(Value::Null);
    };
    movie_url.set_query(None);
    movie_url.set_fragment(None);

    let secure = args.get(2).unwrap_or(&Value::Undefined).coerce_to_boolean();

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
    if let Some(so) = activation.context.avm2_shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    let mut data = None;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name) {
        if let Ok(lso) = flash_lso::read::Reader::default().parse(&saved) {
            data = crate::avm2::amf::deserialize_lso(activation, &lso)?.into();
        }
    }

    let data = if let Some(data) = data {
        data
    } else {
        // No data; create a fresh data object.
        activation
            .avm2()
            .classes()
            .object
            .construct(activation, &[])?
    };

    let created_shared_object =
        SharedObjectObject::from_data_and_name(activation, data, full_name.clone());

    activation
        .context
        .avm2_shared_objects
        .insert(full_name, created_shared_object.into());

    Ok(created_shared_object.into())
}

pub fn get_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let shared_object = this.as_shared_object().unwrap();

    Ok(shared_object.data().into())
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let shared_object = this.as_shared_object().unwrap();

    let data = shared_object.data();
    let name = shared_object.name();

    let mut lso = new_lso(activation, name, data)?;
    // Flash does not write empty LSOs to disk
    if lso.body.is_empty() {
        Ok("flushed".into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        if activation.context.storage.put(name, &bytes) {
            Ok("flushed".into())
        } else {
            Err(Error::AvmError(error(
                activation,
                "Error #2130: Unable to flush SharedObject.",
                2130,
            )?))
        }
    }
    // FIXME - We should dispatch a NetStatusEvent after this function returns
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let shared_object = this.as_shared_object().unwrap();

    let data = shared_object.data();
    let name = shared_object.name();

    let mut lso = new_lso(activation, name, data)?;
    // Flash returns 0 for empty LSOs, but the actual number of bytes (including the header) otherwise
    if lso.body.is_empty() {
        Ok(0.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(bytes.len().into())
    }
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.net.SharedObject", "close");
    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let shared_object = this.as_shared_object().unwrap();

    // Clear the local data object.
    shared_object.reset_data(activation)?;

    // Delete data from storage backend.
    let name = shared_object.name();
    activation.context.storage.remove_key(name);

    Ok(Value::Undefined)
}

pub fn get_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.net.SharedObject", "objectEncoding");
    Ok(0.into())
}

pub fn set_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.net.SharedObject", "objectEncoding");
    Ok(Value::Undefined)
}
