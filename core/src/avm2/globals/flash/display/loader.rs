//! `flash.display.Loader` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::{Error, Object};
use crate::backend::navigator::Request;
use crate::display_object::LoaderDisplay;
use crate::display_object::MovieClip;
use crate::loader::MovieLoaderEventHandler;
use crate::tag_utils::SwfMovie;
use std::sync::Arc;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut this) = this {
        if this.as_display_object().is_none() {
            let new_do = LoaderDisplay::new_with_avm2(activation.context.gc_context, this);
            this.init_display_object(activation.context.gc_context, new_do.into());
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
            &Multiname::new(Namespace::private(""), "_contentLoaderInfo"),
            loader_info.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let url_request = args[0].as_object().unwrap();

        if let Some(context) = args.get(1) {
            if !matches!(context, Value::Null) {
                log::warn!(
                    "Loader.load: 'context' argument is not yet implemented: {:?}",
                    context
                );
            }
        }
        let url = url_request
            .get_property(&Multiname::public("url"), activation)?
            .coerce_to_string(activation)?;

        // This is a dummy MovieClip, which will get overwritten in `Loader`
        let content = MovieClip::new(
            Arc::new(SwfMovie::empty(activation.context.swf.version())),
            activation.context.gc_context,
        );

        let loader_info = this
            .get_property(
                &Multiname::new(Namespace::private(""), "_contentLoaderInfo"),
                activation,
            )?
            .as_object()
            .unwrap();

        let future = activation.context.load_manager.load_movie_into_clip(
            activation.context.player.clone(),
            content.into(),
            // FIXME - set options from the `URLRequest`
            Request::get(url.to_string()),
            Some(url.to_string()),
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}

pub fn load_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let arg0 = args[0].as_object().unwrap();
        let bytearray = arg0.as_bytearray().unwrap();

        if let Some(context) = args.get(1) {
            if !matches!(context, Value::Null) {
                log::warn!(
                    "Loader.load: 'context' argument is not yet implemented: {:?}",
                    context
                );
            }
        }

        // This is a dummy MovieClip, which will get overwritten in `Loader`
        let content = MovieClip::new(
            Arc::new(SwfMovie::empty(activation.context.swf.version())),
            activation.context.gc_context,
        );

        let loader_info = this
            .get_property(
                &Multiname::new(Namespace::private(""), "_contentLoaderInfo"),
                activation,
            )?
            .as_object()
            .unwrap();

        let future = activation.context.load_manager.load_movie_into_clip_bytes(
            activation.context.player.clone(),
            content.into(),
            bytearray.bytes().to_vec(),
            Some(MovieLoaderEventHandler::Avm2LoaderInfo(loader_info)),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}
