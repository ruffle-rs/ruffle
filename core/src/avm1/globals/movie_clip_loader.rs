//! `MovieClipLoader` impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{ArrayBuilder, Object, Value};
use crate::backend::navigator::Request;
use crate::display_object::TDisplayObject;
use crate::loader::MovieLoaderVMData;
use ruffle_macros::istr;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "loadClip" => method(load_clip; DONT_ENUM | DONT_DELETE);
    "unloadClip" => method(unload_clip; DONT_ENUM | DONT_DELETE);
    "getProgress" => method(get_progress; DONT_ENUM | DONT_DELETE);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
    broadcaster_fns: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    broadcaster_fns.initialize(context.strings, class.proto, array_proto);
    class
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let listeners = ArrayBuilder::new(activation).with([this.into()]);
    this.define_value(
        activation.gc(),
        istr!("_listeners"),
        Value::Object(listeners),
        Attribute::DONT_ENUM,
    );
    Ok(Value::Undefined)
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
                    activation.context.player_handle(),
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
        let result = Object::new(&activation.context.strings, None);
        if let Some(target) = target {
            result.define_value(
                activation.gc(),
                istr!("bytesLoaded"),
                target.movie().compressed_len().into(),
                Attribute::empty(),
            );
            result.define_value(
                activation.gc(),
                istr!("bytesTotal"),
                target.movie().compressed_len().into(),
                Attribute::empty(),
            );
        }
        return Ok(result.into());
    }

    Ok(Value::Undefined)
}
