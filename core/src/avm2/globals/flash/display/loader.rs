//! `flash.display.Loader` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::{Error, Object};
use crate::backend::navigator::{NavigationMethod, Request};
use crate::display_object::LoaderDisplay;
use crate::display_object::MovieClip;
use crate::loader::{Avm2LoaderData, MovieLoaderEventHandler};
use crate::tag_utils::SwfMovie;
use std::sync::Arc;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut this) = this {
        if this.as_display_object().is_none() {
            let new_do =
                LoaderDisplay::new_with_avm2(activation, this, activation.context.swf.clone());
            this.init_display_object(&mut activation.context, new_do.into());
        }

        // Some LoaderInfo properties (such as 'bytesLoaded' and 'bytesTotal') are always
        // accessible, even before the 'init' event has fired. Using an empty movie gives
        // us the correct value (0) for them.
        let loader_info = LoaderInfoObject::not_yet_loaded(
            activation,
            Arc::new(SwfMovie::empty(activation.context.swf.version())),
            Some(this),
            None,
            false,
        )?;
        this.set_property(
            &Multiname::new(
                activation.avm2().flash_display_internal,
                "_contentLoaderInfo",
            ),
            loader_info.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let url_request = args.get_object(activation, 0, "request")?;
        let context = args.try_get_object(activation, 1);

        let mut url = url_request
            .get_public_property("url", activation)?
            .coerce_to_string(activation)?
            .to_string();

        // This is a dummy MovieClip, which will get overwritten in `Loader`
        let content = MovieClip::new(
            Arc::new(SwfMovie::empty(activation.context.swf.version())),
            activation.context.gc_context,
        );

        let loader_info = this
            .get_property(
                &Multiname::new(
                    activation.avm2().flash_display_internal,
                    "_contentLoaderInfo",
                ),
                activation,
            )?
            .as_object()
            .unwrap();

        let method = url_request
            .get_public_property("method", activation)?
            .coerce_to_string(activation)?;
        // TODO: URLRequest.method should not be able to have invalid types.
        // We should throw an error there on set.
        let method = NavigationMethod::from_method_str(&method).unwrap_or(NavigationMethod::Get);
        let data = url_request.get_public_property("data", activation)?;
        let body = match (method, data) {
            (_, Value::Null | Value::Undefined) => None,
            (NavigationMethod::Get, data) => {
                // This looks "wrong" but it's Flash-correct.
                // It simply appends the data to the URL if there's already a query,
                // otherwise it adds ?data.
                // This does mean that if there's a #fragment in the URL after the query,
                // the new data gets appended to *that* - which is totally wrong but whatcha gonna do?
                if !url.contains('?') {
                    url.push('?');
                }
                url.push_str(&data.coerce_to_string(activation)?.to_string());
                None
            }
            (NavigationMethod::Post, data) => {
                let content_type = url_request
                    .get_public_property("contentType", activation)?
                    .coerce_to_string(activation)?
                    .to_string();
                if let Some(ba) = data.as_object().and_then(|o| o.as_bytearray_object()) {
                    // Note that this does *not* respect or modify the position.
                    Some((ba.storage().bytes().to_vec(), content_type))
                } else {
                    Some((
                        data.coerce_to_string(activation)?
                            .to_utf8_lossy()
                            .as_bytes()
                            .to_vec(),
                        content_type,
                    ))
                }
            }
        };

        let request = Request::request(method, url.to_string(), body);

        let future = activation.context.load_manager.load_movie_into_clip(
            activation.context.player.clone(),
            content.into(),
            request,
            Some(url.to_string()),
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)),
            Some(Avm2LoaderData {
                context,
                default_domain: activation.caller_domain(),
            }),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}

pub fn load_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let arg0 = args.get_object(activation, 0, "data")?;
        let bytearray = arg0.as_bytearray().unwrap();
        let context = args.try_get_object(activation, 1);

        // This is a dummy MovieClip, which will get overwritten in `Loader`
        let content = MovieClip::new(
            Arc::new(SwfMovie::empty(activation.context.swf.version())),
            activation.context.gc_context,
        );

        let loader_info = this
            .get_property(
                &Multiname::new(
                    activation.avm2().flash_display_internal,
                    "_contentLoaderInfo",
                ),
                activation,
            )?
            .as_object()
            .unwrap();
        let future = activation.context.load_manager.load_movie_into_clip_bytes(
            activation.context.player.clone(),
            content.into(),
            bytearray.bytes().to_vec(),
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)),
            Some(Avm2LoaderData {
                context,
                default_domain: activation.caller_domain(),
            }),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}
