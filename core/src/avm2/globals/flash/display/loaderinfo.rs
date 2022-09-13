//! `flash.display.LoaderInfo` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::Endian;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{loaderinfo_allocator, DomainObject, LoaderStream, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{AvmString, Error};
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};
use std::sync::Arc;
use swf::{write_swf, Compression};

// FIXME - Throw an actual 'Error' with the proper code
const INSUFFICIENT: &str =
    "Error #2099: The loading object is not sufficiently loaded to provide this information.";

/// Implements `flash.display.LoaderInfo`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err("LoaderInfo cannot be constructed".into())
}

/// Implements `flash.display.LoaderInfo`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.LoaderInfo`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// `actionScriptVersion` getter
pub fn action_script_version<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Err(INSUFFICIENT.into());
                }
                LoaderStream::Swf(movie, _) => {
                    let version = if movie.is_action_script_3() { 3 } else { 2 };
                    return Ok(version.into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `applicationDomain` getter
pub fn application_domain<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Ok(DomainObject::from_domain(activation, activation.domain())?.into());
                }
                LoaderStream::Swf(movie, _) => {
                    let domain = activation
                        .context
                        .library
                        .library_for_movie_mut(movie.clone())
                        .avm2_domain();
                    return Ok(DomainObject::from_domain(activation, domain)?.into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `bytesTotal` getter
///
/// TODO: This is also the getter for `bytesLoaded` as we don't yet support
/// streaming loads yet. When we do, we'll need another property for this.
pub fn bytes_total<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(swf) => return Ok(swf.compressed_len().into()),
                LoaderStream::Swf(movie, _) => {
                    return Ok(movie.compressed_len().into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `content` getter
pub fn content<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(swf) => {
                    if Arc::ptr_eq(swf, activation.context.swf) {
                        return Ok(activation.context.stage.root_clip().object2());
                    }
                    return Ok(Value::Null);
                }
                LoaderStream::Swf(_, root) => {
                    return Ok(root.object2());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `contentType` getter
pub fn content_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => return Ok(Value::Null),
                LoaderStream::Swf(_, _) => {
                    return Ok("application/x-shockwave-flash".into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `frameRate` getter
pub fn frame_rate<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Err("Error: The stage's loader info does not have a frame rate".into())
                }
                LoaderStream::Swf(root, _) => {
                    return Ok(root.frame_rate().to_f64().into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `height` getter
pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Err("Error: The stage's loader info does not have a height".into())
                }
                LoaderStream::Swf(root, _) => {
                    return Ok(root.height().to_pixels().into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `isURLInaccessible` getter stub
pub fn is_url_inaccessible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

/// `swfVersion` getter
pub fn swf_version<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Err("Error: The stage's loader info does not have a SWF version".into())
                }
                LoaderStream::Swf(root, _) => {
                    return Ok(root.version().into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `url` getter
pub fn url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            let root = match &*loader_stream {
                LoaderStream::NotYetLoaded(swf) => swf,
                LoaderStream::Swf(root, _) => root,
            };

            let url = root.url().map_or(Value::Null, |url| {
                AvmString::new_utf8(activation.context.gc_context, url).into()
            });
            return Ok(url);
        }
    }

    Ok(Value::Undefined)
}

/// `width` getter
pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => {
                    return Err("Error: The stage's loader info does not have a width".into())
                }
                LoaderStream::Swf(root, _) => {
                    return Ok(root.width().to_pixels().into());
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// `bytes` getter
pub fn bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            let root = match &*loader_stream {
                LoaderStream::NotYetLoaded(swf) => swf,
                LoaderStream::Swf(root, _) => root,
            };

            if root.data().is_empty() {
                return Ok(Value::Null);
            }

            let ba_class = activation.context.avm2.classes().bytearray;

            let ba = ba_class.construct(activation, &[])?;
            let mut ba_write = ba.as_bytearray_mut(activation.context.gc_context).unwrap();

            // First, write a fake header corresponding to an
            // uncompressed SWF
            let mut header = root.header().swf_header().clone();
            header.compression = Compression::None;

            write_swf(&header, &[], &mut *ba_write).unwrap();

            // `swf` always writes an implicit end tag, let's cut that
            // off. We scroll back 2 bytes before writing the actual
            // datastream as it is guaranteed to at least be as long as
            // the implicit end tag we want to get rid of.
            let correct_header_length = ba_write.len() - 2;
            ba_write.set_position(correct_header_length);
            ba_write.write_bytes(root.data())?;

            // `swf` wrote the wrong length (since we wrote the data
            // ourselves), so we need to overwrite it ourselves.
            ba_write.set_position(4);
            ba_write.set_endian(Endian::Little);
            ba_write.write_unsigned_int((root.data().len() + correct_header_length) as u32)?;

            // Finally, reset the array to the correct state.
            ba_write.set_position(0);
            ba_write.set_endian(Endian::Big);

            return Ok(ba.into());
        }
    }

    Ok(Value::Undefined)
}

/// `loader` getter
pub fn loader<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(loader_info) = this.as_ref().and_then(|this| this.as_loader_info_object()) {
        Ok(loader_info.loader().map_or(Value::Null, |v| v.into()))
    } else {
        Ok(Value::Undefined)
    }
}

/// `loaderURL` getter
pub fn loader_url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            let root = match &*loader_stream {
                LoaderStream::NotYetLoaded(swf) => swf,
                LoaderStream::Swf(root, _) => root,
            };

            let loader_url = root.loader_url().or_else(|| root.url()).unwrap_or("");
            return Ok(AvmString::new_utf8(activation.context.gc_context, loader_url).into());
        }
    }

    Ok(Value::Undefined)
}

/// `parameters` getter
pub fn parameters<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(loader_stream) = this
            .as_loader_info_object()
            .and_then(|o| o.as_loader_stream())
        {
            let root = match &*loader_stream {
                LoaderStream::NotYetLoaded(_) => activation.context.swf,
                LoaderStream::Swf(root, _) => root,
            };

            let mut params_obj = activation
                .avm2()
                .classes()
                .object
                .construct(activation, &[])?;
            let parameters = root.parameters();

            for (k, v) in parameters.iter() {
                let avm_k = AvmString::new_utf8(activation.context.gc_context, k);
                let avm_v = AvmString::new_utf8(activation.context.gc_context, v);
                params_obj.set_property(&Multiname::public(avm_k), avm_v.into(), activation)?;
            }

            return Ok(params_obj.into());
        }
    }

    Ok(Value::Undefined)
}

/// `sharedEvents` getter
pub fn shared_events<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(loader_info) = this.as_ref().and_then(|this| this.as_loader_info_object()) {
        return Ok(loader_info.shared_events().into());
    }
    Ok(Value::Undefined)
}

/// Construct `LoaderInfo`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "LoaderInfo"),
        Some(Multiname::new(
            Namespace::package("flash.events"),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<LoaderInfo instance initializer>", mc),
        Method::from_builtin(class_init, "<LoaderInfo class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(loaderinfo_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<LoaderInfo native instance initializer>",
        mc,
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("actionScriptVersion", Some(action_script_version), None),
        ("applicationDomain", Some(application_domain), None),
        ("bytesLoaded", Some(bytes_total), None),
        ("bytesTotal", Some(bytes_total), None),
        ("content", Some(content), None),
        ("contentType", Some(content_type), None),
        ("frameRate", Some(frame_rate), None),
        ("height", Some(height), None),
        ("isURLInaccessible", Some(is_url_inaccessible), None),
        ("swfVersion", Some(swf_version), None),
        ("url", Some(url), None),
        ("width", Some(width), None),
        ("bytes", Some(bytes), None),
        ("loader", Some(loader), None),
        ("loaderURL", Some(loader_url), None),
        ("parameters", Some(parameters), None),
        ("sharedEvents", Some(shared_events), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
