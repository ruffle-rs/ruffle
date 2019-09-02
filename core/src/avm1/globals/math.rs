use crate::avm1::{ActionContext, Object, Value};
use gc_arena::{GcCell, MutationContext};
use std::f64::NAN;

fn abs<'gc>(
    _context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(input) = args.get(0) {
        Value::Number(input.as_number().abs())
    } else {
        Value::Number(NAN)
    }
}

fn round<'gc>(
    _context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(input) = args.get(0) {
        Value::Number(input.as_number().round())
    } else {
        Value::Number(NAN)
    }
}

pub fn create<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Object<'gc>> {
    let mut math = Object::object(gc_context);

    math.set_function("abs", abs, gc_context);
    math.set_function("round", round, gc_context);

    GcCell::allocate(gc_context, math)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::audio::NullAudioBackend;
    use crate::display_object::DisplayObject;
    use crate::movie_clip::MovieClip;
    use gc_arena::rootless_arena;

    fn with_avm<F>(test: F)
    where
        F: FnOnce(&mut ActionContext),
    {
        rootless_arena(|gc_context| {
            let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
            let mut context = ActionContext {
                gc_context,
                global_time: 0,
                root,
                start_clip: root,
                active_clip: root,
                audio: &mut NullAudioBackend::new(),
            };
            test(&mut context);
        });
    }

    #[test]
    fn test_abs_nan() {
        with_avm(|context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(abs(context, *math.read(), &[]), Value::Number(NAN));
            assert_eq!(
                abs(context, *math.read(), &[Value::Number(NAN)]),
                Value::Number(NAN)
            );
            assert_eq!(
                abs(context, *math.read(), &[Value::String("".to_string())]),
                Value::Number(NAN)
            );
        });
    }

    #[test]
    fn test_abs_valid() {
        with_avm(|context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(
                abs(context, *math.read(), &[Value::Number(-50.0)]),
                Value::Number(50.0)
            );
            assert_eq!(
                abs(context, *math.read(), &[Value::Number(50.0)]),
                Value::Number(50.0)
            );
            assert_eq!(
                abs(context, *math.read(), &[Value::Bool(true)]),
                Value::Number(1.0)
            );
            assert_eq!(
                abs(context, *math.read(), &[Value::String("-10".to_string())]),
                Value::Number(10.0)
            );
        });
    }

    #[test]
    fn test_round_nan() {
        with_avm(|context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(round(context, *math.read(), &[]), Value::Number(NAN));
            assert_eq!(
                round(context, *math.read(), &[Value::Number(NAN)]),
                Value::Number(NAN)
            );
            assert_eq!(
                round(context, *math.read(), &[Value::String("".to_string())]),
                Value::Number(NAN)
            );
        });
    }

    #[test]
    fn test_round_valid() {
        with_avm(|context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(
                round(context, *math.read(), &[Value::Number(0.4)]),
                Value::Number(0.0)
            );
            assert_eq!(
                round(context, *math.read(), &[Value::Number(1.5)]),
                Value::Number(2.0)
            );
            assert_eq!(
                round(context, *math.read(), &[Value::String("-5.4".to_string())]),
                Value::Number(-5.0)
            );
        });
    }
}
