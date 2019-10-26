use crate::avm1::function::Executable;
use crate::avm1::object::{Attribute::*, Object};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, UpdateContext, Value};
use crate::display_object::{DisplayNode, DisplayObject, MovieClip};
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, _context, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.read().as_movie_clip() {
                            return Ok(ReturnValue::Immediate($fn(movie_clip, args)));
                        }
                    }
                    Ok(ReturnValue::Immediate(Value::Undefined))
                },
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
            );
        )*
    }};
}

macro_rules! with_movie_clip_mut {
    ( $gc_context: ident, $object:ident, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.write(context.gc_context).as_movie_clip_mut() {
                            return Ok(ReturnValue::Immediate($fn(movie_clip, context, display_object, args)));
                        }
                    }
                    Ok(ReturnValue::Immediate(Value::Undefined))
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
            );
        )*
    }};
}

pub fn overwrite_root<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.write(ac.gc_context)
        .force_set("_root", new_val, EnumSet::new());

    Ok(ReturnValue::Immediate(Value::Undefined))
}

pub fn overwrite_global<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.write(ac.gc_context)
        .force_set("_global", new_val, EnumSet::new());

    Ok(ReturnValue::Immediate(Value::Undefined))
}

pub fn create_movie_object<'gc>(gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
    let mut object = Object::object(gc_context);

    with_movie_clip_mut!(
        gc_context,
        object,
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
        "getBytesLoaded" => |_movie_clip: &MovieClip<'gc>, _args| {
            // TODO find a correct value
            Value::Number(1.0)
        },
        "getBytesTotal" => |_movie_clip: &MovieClip<'gc>, _args| {
            // TODO find a correct value
            Value::Number(1.0)
        },
        "toString" => |movie_clip: &MovieClip, _args| {
            Value::String(movie_clip.name().to_string())
        }
    );

    object.force_set_virtual(
        "_global",
        Executable::Native(|avm, context, _this, _args| {
            Ok(ReturnValue::Immediate(avm.global_object(context)))
        }),
        Some(Executable::Native(overwrite_global)),
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_root",
        Executable::Native(|avm, context, _this, _args| {
            Ok(ReturnValue::Immediate(avm.root_object(context)))
        }),
        Some(Executable::Native(overwrite_root)),
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_parent",
        Executable::Native(|_avm, _context, this, _args| {
            Ok(ReturnValue::Immediate(
                this.read()
                    .display_node()
                    .and_then(|mc| mc.read().parent())
                    .and_then(|dn| dn.read().object().as_object().ok())
                    .map(|o| Value::Object(o.to_owned()))
                    .unwrap_or(Value::Undefined),
            ))
        }),
        None,
        EnumSet::new(),
    );

    object
}
