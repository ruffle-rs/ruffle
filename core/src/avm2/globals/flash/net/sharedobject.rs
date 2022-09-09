//! `flash.net.SharedObject` builtin/prototype

use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::TObject;
use crate::avm2::traits::Trait;
use crate::avm2::Multiname;
use crate::avm2::{Activation, Error, Namespace, Object, QName, Value};
use crate::display_object::DisplayObject;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use flash_lso::types::{AMFVersion, Lso};
use gc_arena::{GcCell, MutationContext};
use std::borrow::Cow;

fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let data = activation
            .context
            .avm2
            .classes()
            .object
            .construct(activation, &[])?;
        this.set_property(&Multiname::public("data"), data.into(), activation)?;
    }

    Ok(Value::Undefined)
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

    let movie = if let DisplayObject::MovieClip(movie) = activation.context.stage.root_clip() {
        movie
    } else {
        log::error!("SharedObject::get_local: Movie was None");
        return Ok(Value::Null);
    };

    let mut movie_url = if let Some(url) = movie.movie().and_then(|m| m.url().map(|u| u.to_owned()))
    {
        if let Ok(url) = url::Url::parse(&url) {
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

    let secure = args.get(2).unwrap_or(&Value::Undefined).coerce_to_boolean();

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
    if let Some(so) = activation.context.avm2_shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    // Data property only should exist when created with getLocal/Remote
    let constructor = activation.avm2().classes().sharedobject;
    let mut this = constructor.construct(activation, &[])?;

    // Set the internal name
    let ruffle_name = Multiname::new(Namespace::Private("".into()), "_ruffleName");
    this.set_property(
        &ruffle_name,
        AvmString::new_utf8(activation.context.gc_context, &full_name).into(),
        activation,
    )?;

    let mut data = Value::Undefined;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name) {
        if let Ok(lso) = flash_lso::read::Reader::default().parse(&saved) {
            data = crate::avm2::amf::deserialize_lso(activation, &lso)?.into();
        }
    }

    if data == Value::Undefined {
        // No data; create a fresh data object.
        data = activation
            .avm2()
            .classes()
            .object
            .construct(activation, &[])?
            .into();
    }

    this.set_property(&Multiname::public("data"), data, activation)?;
    activation
        .context
        .avm2_shared_objects
        .insert(full_name, this);

    Ok(this.into())
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let data = this
            .get_property(&Multiname::public("data"), activation)?
            .coerce_to_object(activation)?;

        let ruffle_name = Multiname::new(Namespace::Private("".into()), "_ruffleName");
        let name = this
            .get_property(&ruffle_name, activation)?
            .coerce_to_string(activation)?;
        let name = name.to_utf8_lossy();

        let mut elements = Vec::new();
        crate::avm2::amf::recursive_serialize(activation, data, &mut elements)?;
        let mut lso = Lso::new(
            elements,
            &name
                .split('/')
                .last()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            AMFVersion::AMF3,
        );

        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();

        return Ok(activation.context.storage.put(&name, &bytes).into());
    }
    Ok(Value::Undefined)
}

/// Construct `SharedObject`'s class.
/// NOTE: We currently always use AMF3 serialization.
/// If you implement the `defaultObjectEncoding` or `objectEncoding`,
/// you will need to adjust the serialization and deserialization code
/// to work with AMF0.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.net"), "SharedObject"),
        Some(Multiname::new(
            Namespace::package("flash.events"),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<SharedObject instance initializer>", mc),
        Method::from_builtin(class_init, "<SharedObject class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "data"),
        Multiname::public("Object"),
        None,
    ));

    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(""), "_ruffleName"),
        Multiname::public("String"),
        None,
    ));

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("getLocal", get_local)];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("flush", flush)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);
    class
}
