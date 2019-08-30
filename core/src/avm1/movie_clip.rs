use crate::avm1::object::Object;
use crate::avm1::Value;
use crate::movie_clip::MovieClip;
use gc_arena::MutationContext;

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.set_function(
                $name,
                |_gc_context, this, args| -> Value<'gc> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.read().as_movie_clip() {
                            return $fn(movie_clip, args);
                        }
                    }
                    Value::Undefined
                },
                $gc_context,
            );
        )*
    }};
}

macro_rules! with_movie_clip_mut {
    ( $gc_context: ident, $object:ident, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.set_function(
                $name,
                |gc_context, this, args| -> Value<'gc> {
                    if let Some(display_object) = this.read().display_node() {
                        if let Some(movie_clip) = display_object.write(gc_context).as_movie_clip_mut() {
                            return $fn(movie_clip, args);
                        }
                    }
                    Value::Undefined
                },
                $gc_context,
            );
        )*
    }};
}

pub fn create_movie_object<'gc>(gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
    let mut object = Object::new();

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
            Value::Number(0.0)
        },
        "getBytesTotal" => |_movie_clip: &MovieClip, _args| {
            // TODO find a correct value
            Value::Number(0.0)
        }
    );

    object
}
