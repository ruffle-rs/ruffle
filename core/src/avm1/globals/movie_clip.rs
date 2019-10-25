//! MovieClip prototype

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ObjectCell, ScriptObject, UpdateContext, Value};
use crate::display_object::{DisplayNode, DisplayObject, MovieClip};
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};

/// Implements `MovieClip`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: ObjectCell<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, _context, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.read().as_display_node() {
                        if let Some(movie_clip) = display_object.read().as_movie_clip() {
                            return Ok($fn(movie_clip, args));
                        }
                    }
                    Ok(Value::Undefined.into())
                },
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

macro_rules! with_movie_clip_mut {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.read().as_display_node() {
                        if let Some(movie_clip) = display_object.write(context.gc_context).as_movie_clip_mut() {
                            return Ok($fn(movie_clip, context, display_object, args).into());
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
    this: ObjectCell<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.write(ac.gc_context)
        .as_script_object_mut()
        .unwrap()
        .force_set("_root", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn overwrite_global<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: ObjectCell<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.write(ac.gc_context)
        .as_script_object_mut()
        .unwrap()
        .force_set("_global", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: ObjectCell<'gc>,
    fn_proto: ObjectCell<'gc>,
) -> ObjectCell<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    with_movie_clip_mut!(
        gc_context,
        object,
        Some(fn_proto),
        "nextFrame" => |movie_clip: &mut MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayNode<'gc>, _args| {
            movie_clip.next_frame(context);
            Value::Undefined
        },
        "prevFrame" =>  |movie_clip: &mut MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayNode<'gc>, _args| {
            movie_clip.prev_frame(context);
            Value::Undefined
        },
        "play" => |movie_clip: &mut MovieClip<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayNode<'gc>, _args| {
            movie_clip.play();
            Value::Undefined
        },
        "stop" => |movie_clip: &mut MovieClip<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _cell: DisplayNode<'gc>, _args| {
            movie_clip.stop(context);
            Value::Undefined
        }
    );

    with_movie_clip!(
        gc_context,
        object,
        Some(fn_proto),
        "getBytesLoaded" => |_movie_clip: &MovieClip<'gc>, _args| {
            // TODO find a correct value
            1.0.into()
        },
        "getBytesTotal" => |_movie_clip: &MovieClip<'gc>, _args| {
            // TODO find a correct value
            1.0.into()
        },
        "toString" => |movie_clip: &MovieClip, _args| {
            movie_clip.name().to_string().into()
        }
    );

    object.force_set_virtual(
        "_global",
        Executable::Native(|avm, context, _this, _args| Ok(avm.global_object(context).into())),
        Some(Executable::Native(overwrite_global)),
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_root",
        Executable::Native(|avm, context, _this, _args| Ok(avm.root_object(context).into())),
        Some(Executable::Native(overwrite_root)),
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_parent",
        Executable::Native(|_avm, _context, this, _args| {
            Ok(this
                .read()
                .as_display_node()
                .and_then(|mc| mc.read().parent())
                .and_then(|dn| dn.read().object().as_object().ok())
                .map(|o| Value::Object(o.to_owned()))
                .unwrap_or(Value::Undefined)
                .into())
        }),
        None,
        EnumSet::new(),
    );

    GcCell::allocate(gc_context, Box::new(object))
}
