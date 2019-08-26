use crate::avm1::Value;
use crate::builtins::Object;
use gc_arena::{GcCell, MutationContext};

pub fn create_movie_clip<'gc>(gc_context: MutationContext<'gc, '_>) -> Value<'gc> {
    let mut class = Object::new();

    class.set(
        "getBytesTotal",
        Value::NativeFunction(|_args: &[Value<'gc>]| Value::Number(1.0)),
    );

    class.set(
        "getBytesLoaded",
        Value::NativeFunction(|_args: &[Value<'gc>]| Value::Number(1.0)),
    );

    Value::Object(GcCell::allocate(gc_context, class))
}
