//! `flash.display.LoaderInfo` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::Endian;
use crate::avm2::error::make_error_2099;
use crate::avm2::object::{DomainObject, LoaderStream, ScriptObject, TObject as _};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use crate::loader::ContentType;
use crate::string::AvmString;
use crate::{avm2_stub_getter, avm2_stub_method};
use std::sync::Arc;
use swf::{write_swf, Compression};
use url::Url;

/// `actionScriptVersion` getter
pub fn get_action_script_version<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(movie, _) => {
                let version = if movie.is_action_script_3() { 3 } else { 2 };
                return Ok(version.into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `applicationDomain` getter
pub fn get_application_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(movie, _, _) => {
                let domain = activation
                    .context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .try_avm2_domain();

                if let Some(domain) = domain {
                    return Ok(DomainObject::from_domain(activation, domain).into());
                } else {
                    return Ok(Value::Null);
                }
            }

            // A loaded SWF will always have an AVM2 domain present.
            LoaderStream::Swf(movie, _) => {
                let domain = activation
                    .context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .avm2_domain();
                return Ok(DomainObject::from_domain(activation, domain).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `bytesTotal` getter
pub fn get_bytes_total<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(swf, _, _) => return Ok(swf.compressed_len().into()),
            LoaderStream::Swf(movie, _) => {
                return Ok(movie.compressed_len().into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `bytesLoaded` getter
pub fn get_bytes_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let loader_info = this.as_loader_info_object().unwrap();
    let loader_stream = loader_info.loader_stream();
    match &*loader_stream {
        LoaderStream::NotYetLoaded(swf, None, _) => {
            if loader_info.errored() {
                return Ok(swf.compressed_len().into());
            }
            Ok(0.into())
        }
        LoaderStream::Swf(swf, root) | LoaderStream::NotYetLoaded(swf, Some(root), _) => {
            if root.as_bitmap().is_some() {
                return Ok(swf.compressed_len().into());
            }
            Ok(root
                .as_movie_clip()
                .map(|mc| mc.compressed_loaded_bytes())
                .unwrap_or_default()
                .into())
        }
    }
}

/// `content` getter
pub fn get_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let loader_info = this.as_loader_info_object().unwrap();
    if !loader_info.expose_content() {
        return Ok(Value::Null);
    }

    let loader_stream = loader_info.loader_stream();
    match &*loader_stream {
        LoaderStream::Swf(_, root) | LoaderStream::NotYetLoaded(_, Some(root), _) => {
            Ok(root.object2_or_null())
        }
        _ => Ok(Value::Null),
    }
}

/// `contentType` getter
pub fn get_content_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_info) = this.as_loader_info_object() {
        let content_type = match loader_info.content_type_hide_before_init() {
            ContentType::Swf => "application/x-shockwave-flash",
            ContentType::Jpeg => "image/jpeg",
            ContentType::Png => "image/png",
            ContentType::Gif => "image/gif",
            ContentType::Unknown => return Ok(Value::Null),
        };

        return Ok(AvmString::new_utf8(activation.gc(), content_type).into());
    }

    Ok(Value::Undefined)
}

/// `frameRate` getter
pub fn get_frame_rate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(root, _) => {
                return Ok(root.frame_rate().to_f64().into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `height` getter
pub fn get_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(root, _) => {
                return Ok(root.height().to_pixels().into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `isURLInaccessible` getter
pub fn get_is_url_inaccessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.LoaderInfo", "isURLInaccessible");
    Ok(false.into())
}

/// `sameDomain` getter
pub fn get_same_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(_root, _) => {
                avm2_stub_getter!(activation, "flash.display.LoaderInfo", "sameDomain");
                return Ok(false.into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `childAllowsParent` getter
pub fn get_child_allows_parent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let loader_info = this.as_loader_info_object().unwrap();
    let loader_stream = loader_info.loader_stream();
    match &*loader_stream {
        LoaderStream::NotYetLoaded(_, _, _) => Err(make_error_2099(activation)),
        LoaderStream::Swf(root, dobj) => {
            // TODO: respect allowDomain() and polices.
            avm2_stub_getter!(activation, "flash.display.LoaderInfo", "childAllowsParent");

            if let Some(loader) = loader_info.loader() {
                let loader = loader.as_display_object().expect("Loader is a DO");
                let parent_movie = loader.movie();

                if let Ok(child_url) = Url::parse(root.url()) {
                    if let Ok(parent_url) = Url::parse(parent_movie.url()) {
                        if child_url.host() == parent_url.host() {
                            return Ok(true.into());
                        }
                    }
                }
                Ok(false.into())
            } else {
                // Only the root movie is LoaderStream::Swf but missing a loader.
                // In that case, return true.
                assert!(
                    Arc::ptr_eq(root, activation.context.root_swf)
                        && dobj.as_movie_clip().is_some()
                );
                Ok(true.into())
            }
        }
    }
}

/// `parentAllowsChild` getter
pub fn get_parent_allows_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let loader_info = this.as_loader_info_object().unwrap();
    let loader_stream = loader_info.loader_stream();
    match &*loader_stream {
        LoaderStream::NotYetLoaded(_, _, _) => Err(make_error_2099(activation)),
        LoaderStream::Swf(root, dobj) => {
            // TODO: respect allowDomain() and polices.
            avm2_stub_getter!(activation, "flash.display.LoaderInfo", "parentAllowsChild");

            if let Some(loader) = loader_info.loader() {
                let loader = loader.as_display_object().expect("Loader is a DO");
                let parent_movie = loader.movie();

                if let Ok(child_url) = Url::parse(root.url()) {
                    if let Ok(parent_url) = Url::parse(parent_movie.url()) {
                        if child_url.host() == parent_url.host() {
                            return Ok(true.into());
                        }
                    }
                }
                Ok(false.into())
            } else {
                // See comment on childAllowsParent
                assert!(
                    Arc::ptr_eq(root, activation.context.root_swf)
                        && dobj.as_movie_clip().is_some()
                );
                Ok(true.into())
            }
        }
    }
}

/// `swfVersion` getter
pub fn get_swf_version<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(root, _) => {
                return Ok(root.version().into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `url` getter
pub fn get_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_info) = this.as_loader_info_object() {
        if !loader_info.expose_content() {
            return Ok(Value::Null);
        }

        let loader_stream = loader_info.loader_stream();
        let root = match &*loader_stream {
            LoaderStream::NotYetLoaded(root, _, _) | LoaderStream::Swf(root, _) => root,
        };
        return Ok(AvmString::new_utf8(activation.gc(), root.url()).into());
    }

    Ok(Value::Undefined)
}

/// `width` getter
pub fn get_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        match &*loader_stream {
            LoaderStream::NotYetLoaded(_, _, _) => {
                return Err(make_error_2099(activation));
            }
            LoaderStream::Swf(root, _) => {
                return Ok(root.width().to_pixels().into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// `bytes` getter
pub fn get_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let loader_info = this.as_loader_info_object().unwrap();
    let loader_stream = loader_info.loader_stream();
    let (root, dobj) = match &*loader_stream {
        LoaderStream::NotYetLoaded(_, None, _) => {
            if loader_info.errored() {
                return activation
                    .avm2()
                    .classes()
                    .bytearray
                    .construct(activation, &[]);
            }
            // If we haven't even started loading yet (we have no root clip),
            // then return null. FIXME - we should probably store the ByteArray
            // in a field, and initialize it when we start loading.
            return Ok(Value::Null);
        }
        LoaderStream::NotYetLoaded(swf, Some(dobj), _) => (swf, dobj),
        LoaderStream::Swf(root, dobj) => (root, dobj),
    };

    let ba = activation
        .avm2()
        .classes()
        .bytearray
        .construct(activation, &[])?
        .as_object()
        .unwrap();

    if root.data().is_empty() {
        return Ok(ba.into());
    }

    if dobj.as_bitmap().is_some() {
        // TODO - we need to construct a fake SWF that contains a 'Define' tag for the image data.
        avm2_stub_method!(
            activation,
            "flash.display.LoaderInfo",
            "bytes",
            "with image"
        );
    }

    let mut ba_write = ba.as_bytearray_mut().unwrap();

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
    ba_write
        .write_bytes(root.data())
        .map_err(|e| e.to_avm(activation))?;

    // `swf` wrote the wrong length (since we wrote the data
    // ourselves), so we need to overwrite it ourselves.
    ba_write.set_position(4);
    ba_write.set_endian(Endian::Little);
    ba_write
        .write_unsigned_int((root.data().len() + correct_header_length) as u32)
        .map_err(|e| e.to_avm(activation))?;

    // Finally, reset the array to the correct state.
    ba_write.set_position(0);
    ba_write.set_endian(Endian::Big);

    Ok(ba.into())
}

/// `loader` getter
pub fn get_loader<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_info) = this.as_loader_info_object() {
        Ok(loader_info.loader().map_or(Value::Null, |v| v.into()))
    } else {
        Ok(Value::Undefined)
    }
}

/// `loaderURL` getter
pub fn get_loader_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        let root = match &*loader_stream {
            LoaderStream::NotYetLoaded(swf, _, _) => swf,
            LoaderStream::Swf(root, _) => root,
        };

        let loader_url = root.loader_url().unwrap_or_else(|| root.url());
        return Ok(AvmString::new_utf8(activation.gc(), loader_url).into());
    }

    Ok(Value::Undefined)
}

/// `parameters` getter
pub fn get_parameters<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_stream) = this.as_loader_info_object().map(|o| o.loader_stream()) {
        let root = match &*loader_stream {
            LoaderStream::NotYetLoaded(root, _, _) => root,
            LoaderStream::Swf(root, _) => root,
        };

        let params_obj = ScriptObject::new_object(activation.context);
        let parameters = root.parameters();

        for (k, v) in parameters.iter() {
            let avm_k = AvmString::new_utf8(activation.gc(), k);
            let avm_v = AvmString::new_utf8(activation.gc(), v);
            params_obj.set_dynamic_property(avm_k, avm_v.into(), activation.gc());
        }

        return Ok(params_obj.into());
    }

    Ok(Value::Undefined)
}

/// `sharedEvents` getter
pub fn get_shared_events<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_info) = this.as_loader_info_object() {
        return Ok(loader_info.shared_events().into());
    }
    Ok(Value::Undefined)
}

/// `uncaughtErrorEvents` getter
pub fn get_uncaught_error_events<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(loader_info) = this.as_loader_info_object() {
        return Ok(loader_info.uncaught_error_events().into());
    }
    Ok(Value::Undefined)
}
