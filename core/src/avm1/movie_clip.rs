use crate::avm1::object::{Attribute::*, Object};
use crate::avm1::Value;
use crate::movie_clip::MovieClip;
use enumset::EnumSet;
use gc_arena::MutationContext;

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, _context, this, args| -> Value<'gc> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.read().as_movie_clip() {
                            return $fn(movie_clip, args);
                        }
                    }
                    Value::Undefined
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
                |_avm, context, this, args| -> Value<'gc> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.write(context.gc_context).as_movie_clip_mut() {
                            return $fn(movie_clip, args);
                        }
                    }
                    Value::Undefined
                },
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
            );
        )*
    }};
}

pub fn create_movie_object<'gc>(gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
    let mut object = Object::object(gc_context);

    with_movie_clip_mut!(
        gc_context,
        object,
        "nextFrame" => |movie_clip: &mut MovieClip, _args| {
            movie_clip.next_frame();
            Value::Undefined
        },
        "prevFrame" => |movie_clip: &mut MovieClip, _args| {
            movie_clip.prev_frame();
            Value::Undefined
        },
        "play" => |movie_clip: &mut MovieClip, _args| {
            movie_clip.play();
            Value::Undefined
        },
        "stop" => |movie_clip: &mut MovieClip, _args| {
            movie_clip.stop();
            Value::Undefined
        }
    );

    with_movie_clip!(
        gc_context,
        object,
        "getBytesLoaded" => |_movie_clip: &MovieClip, _args| {
            // TODO find a correct value
            Value::Number(1.0)
        },
        "getBytesTotal" => |_movie_clip: &MovieClip, _args| {
            // TODO find a correct value
            Value::Number(1.0)
        }
    );

    object.force_set_virtual(
        "_global",
        |avm, context, _this, _args| avm.global_object(context),
        None,
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_root",
        |avm, context, _this, _args| avm.root_object(context),
        None,
        EnumSet::new(),
    );

    object.force_set_virtual(
        "_parent",
        |_avm, _context, this, _args| {
            this.read()
                .display_node()
                .and_then(|mc| mc.read().parent())
                .and_then(|dn| dn.read().object().as_object().ok())
                .map(|o| Value::Object(o.to_owned()))
                .unwrap_or(Value::Undefined)
        },
        None,
        EnumSet::new(),
    );

    object
}
