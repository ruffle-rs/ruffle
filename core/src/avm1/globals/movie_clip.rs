//! MovieClip prototype

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{DisplayObject, MovieClip, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;

/// Implements `MovieClip`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            let ret: ReturnValue<'gc> = $fn(movie_clip, context, display_object, args);
                            return Ok(ret);
                        }
                    }
                    Ok(Value::Undefined.into())
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn overwrite_root<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_root", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn overwrite_global<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_global", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    with_movie_clip!(
        gc_context,
        object,
        Some(fn_proto),
        "nextFrame" => |movie_clip: MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            movie_clip.next_frame(context);
            Value::Undefined.into()
        },
        "prevFrame" => |movie_clip: MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            movie_clip.prev_frame(context);
            Value::Undefined.into()
        },
        "play" => |movie_clip: MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            movie_clip.play(context);
            Value::Undefined.into()
        },
        "stop" => |movie_clip: MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            movie_clip.stop(context);
            Value::Undefined.into()
        },
        "getBytesLoaded" => |_movie_clip: MovieClip<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            // TODO find a correct value
            1.0.into()
        },
        "getBytesTotal" => |_movie_clip: MovieClip<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| {
            // TODO find a correct value
            1.0.into()
        },
        "toString" => |movie_clip: MovieClip<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayObject<'gc>, _args| -> ReturnValue<'gc> {
            movie_clip.path().into()
        }
    );

    object.add_property(
        gc_context,
        "_global",
        Executable::Native(|avm, context, _this, _args| Ok(avm.global_object(context).into())),
        Some(Executable::Native(overwrite_global)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_root",
        Executable::Native(|avm, context, _this, _args| Ok(avm.root_object(context).into())),
        Some(Executable::Native(overwrite_root)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_parent",
        Executable::Native(|_avm, _context, this, _args| {
            log::info!("Self: {:?}", this.as_display_object());
            log::info!("ASDASDASD");
            Ok(this
                .as_display_object()
                .and_then(|mc| mc.parent())
                .and_then(|dn| dn.object().as_object().ok())
                .map(Value::Object)
                .unwrap_or(Value::Undefined)
                .into())
        }),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.into()
}
