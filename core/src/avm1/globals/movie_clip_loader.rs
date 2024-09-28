//! `MovieClipLoader` impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::object::script_object::ScriptObject;
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, Value};
use crate::backend::navigator::Request;
use crate::display_object::TDisplayObject;
use crate::loader::MovieLoaderVMData;
use crate::string::StringContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "loadClip" => method(load_clip; DONT_ENUM | DONT_DELETE);
    "unloadClip" => method(unload_clip; DONT_ENUM | DONT_DELETE);
    "getProgress" => method(get_progress; DONT_ENUM | DONT_DELETE);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let listeners = ArrayObject::new(
        activation.context.gc_context,
        activation.context.avm1.prototypes().array,
        [this.into()],
    );
    this.define_value(
        activation.context.gc_context,
        "_listeners",
        Value::Object(listeners.into()),
        Attribute::DONT_ENUM,
    );
    Ok(this.into())
}

fn load_clip<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [url, target, ..] = args {
        if let Value::String(url) = url {
            let target = match target {
                Value::String(_) => {
                    let start_clip = activation.target_clip_or_root();
                    activation.resolve_target_display_object(start_clip, *target, true)?
                }
                Value::Number(level_id) => {
                    // Levels are rounded down.
                    // TODO: What happens with negative levels?
                    Some(activation.get_or_create_level(*level_id as i32))
                }
                Value::Object(object) => object.as_display_object(),
                Value::MovieClip(_) => target.coerce_to_object(activation).as_display_object(),
                _ => None,
            };
            if let Some(target) = target {
                let future = activation.context.load_manager.load_movie_into_clip(
                    activation.context.player.clone(),
                    target,
                    Request::get(url.to_utf8_lossy().into_owned()),
                    None,
                    MovieLoaderVMData::Avm1 {
                        broadcaster: Some(this),
                    },
                );
                activation.context.navigator.spawn_future(future);

                return Ok(true.into());
            }
        }

        return Ok(false.into());
    }

    Ok(Value::Undefined)
}

fn unload_clip<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [target, ..] = args {
        let target = match target {
            Value::String(_) => {
                let start_clip = activation.target_clip_or_root();
                activation.resolve_target_display_object(start_clip, *target, true)?
            }
            Value::Number(level_id) => {
                // Levels are rounded down.
                // TODO: What happens with negative levels?
                activation.get_level(*level_id as i32)
            }
            Value::Object(object) => object.as_display_object(),
            Value::MovieClip(_) => target.coerce_to_object(activation).as_display_object(),
            _ => None,
        };
        if let Some(target) = target {
            // TODO: Find out what's the correct behaviour. If target isn't a MovieClip,
            // does Flash also wait a frame to execute avm1_unload? Is avm1_unload_movie
            // the correct call?
            if let Some(mc) = target.as_movie_clip() {
                mc.avm1_unload_movie(activation.context);
            } else {
                target.avm1_unload(activation.context);
            }
            return Ok(true.into());
        }

        return Ok(false.into());
    }

    Ok(Value::Undefined)
}

fn get_progress<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [target, ..] = args {
        let target = match target {
            Value::String(_) => {
                let start_clip = activation.target_clip_or_root();
                activation.resolve_target_display_object(start_clip, *target, true)?
            }
            Value::Number(level_id) => {
                // Levels are rounded down.
                // TODO: What happens with negative levels?
                activation.get_level(*level_id as i32)
            }
            Value::Object(object) if object.as_display_object().is_some() => {
                object.as_display_object()
            }
            Value::MovieClip(_) => target.coerce_to_object(activation).as_display_object(),
            _ => return Ok(Value::Undefined),
        };
        let result = ScriptObject::new(activation.context.gc_context, None);
        if let Some(target) = target {
            result.define_value(
                activation.context.gc_context,
                "bytesLoaded",
                target.movie().compressed_len().into(),
                Attribute::empty(),
            );
            result.define_value(
                activation.context.gc_context,
                "bytesTotal",
                target.movie().compressed_len().into(),
                Attribute::empty(),
            );
        }
        return Ok(result.into());
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let mcl_proto = ScriptObject::new(context.gc_context, Some(proto));
    broadcaster_functions.initialize(context.gc_context, mcl_proto.into(), array_proto);
    define_properties_on(PROTO_DECLS, context, mcl_proto, fn_proto);
    mcl_proto.into()
}
