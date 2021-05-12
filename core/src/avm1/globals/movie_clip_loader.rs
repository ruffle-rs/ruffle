//! `MovieClipLoader` impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::object::script_object::ScriptObject;
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, Value};
use crate::backend::navigator::RequestOptions;
use crate::display_object::{DisplayObject, TDisplayObject};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "loadClip" => method(load_clip);
    "unloadClip" => method(unload_clip);
    "getProgress" => method(get_progress);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let listeners = ScriptObject::array(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes().array),
    );
    this.define_value(
        activation.context.gc_context,
        "_listeners",
        Value::Object(listeners.into()),
        Attribute::DONT_ENUM,
    );
    listeners.set_array_element(0, Value::Object(this), activation.context.gc_context);

    Ok(this.into())
}

pub fn load_clip<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);
    let url = url_val.coerce_to_string(activation)?;
    let target = args.get(1).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(target) = target {
        if let Some(movieclip) = target
            .as_display_object()
            .and_then(|dobj| dobj.as_movie_clip())
        {
            let fetch = activation
                .context
                .navigator
                .fetch(&url, RequestOptions::get());
            let process = activation.context.load_manager.load_movie_into_clip(
                activation.context.player.clone().unwrap(),
                DisplayObject::MovieClip(movieclip),
                fetch,
                url.to_string(),
                None,
                Some(this),
            );

            activation.context.navigator.spawn_future(process);
        }

        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

pub fn unload_clip<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let target = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(target) = target {
        if let Some(mut movieclip) = target
            .as_display_object()
            .and_then(|dobj| dobj.as_movie_clip())
        {
            movieclip.unload(&mut activation.context);
            movieclip.replace_with_movie(activation.context.gc_context, None);

            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn get_progress<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let target = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(target) = target {
        if let Some(movieclip) = target
            .as_display_object()
            .and_then(|dobj| dobj.as_movie_clip())
        {
            let ret_obj = ScriptObject::object(activation.context.gc_context, None);
            ret_obj.define_value(
                activation.context.gc_context,
                "bytesLoaded",
                movieclip
                    .movie()
                    .map(|mv| (mv.header().uncompressed_length).into())
                    .unwrap_or(Value::Undefined),
                Attribute::empty(),
            );
            ret_obj.define_value(
                activation.context.gc_context,
                "bytesTotal",
                movieclip
                    .movie()
                    .map(|mv| (mv.header().uncompressed_length).into())
                    .unwrap_or(Value::Undefined),
                Attribute::empty(),
            );

            return Ok(ret_obj.into());
        }
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let mcl_proto = ScriptObject::object(gc_context, Some(proto));
    broadcaster_functions.initialize(gc_context, mcl_proto.into(), array_proto);
    define_properties_on(PROTO_DECLS, gc_context, mcl_proto, fn_proto);
    mcl_proto.into()
}
