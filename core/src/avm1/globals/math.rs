use crate::avm1::{ActionContext, Avm1, Object, Value};
use gc_arena::{GcCell, MutationContext};
use rand::Rng;
use std::f64::NAN;

macro_rules! wrap_std {
    ( $object: ident, $gc_context: ident, $($name:expr => $std:path),* ) => {{
        $(
            $object.set_function(
                $name,
                |_avm, _context, _this, args| -> Value<'gc> {
                    if let Some(input) = args.get(0) {
                        Value::Number($std(input.as_number()))
                    } else {
                        Value::Number(NAN)
                    }
                },
                $gc_context,
            );
        )*
    }};
}

fn atan2<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            return Value::Number(y.as_number().atan2(x.as_number()));
        } else {
            return Value::Number(y.as_number().atan2(0.0));
        }
    }
    Value::Number(NAN)
}

pub fn random<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    _args: &[Value<'gc>],
) -> Value<'gc> {
    Value::Number(action_context.rng.gen_range(0.0f64, 1.0f64))
}

pub fn create<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Object<'gc>> {
    let mut math = Object::object(gc_context);

    math.set("E", Value::Number(std::f64::consts::E));
    math.set("LN10", Value::Number(std::f64::consts::LN_10));
    math.set("LN2", Value::Number(std::f64::consts::LN_2));
    math.set("LOG10E", Value::Number(std::f64::consts::LOG10_E));
    math.set("LOG2E", Value::Number(std::f64::consts::LOG2_E));
    math.set("PI", Value::Number(std::f64::consts::PI));
    math.set("SQRT1_2", Value::Number(std::f64::consts::FRAC_1_SQRT_2));
    math.set("SQRT2", Value::Number(std::f64::consts::SQRT_2));

    wrap_std!(math, gc_context,
        "abs" => f64::abs,
        "acos" => f64::acos,
        "asin" => f64::asin,
        "atan" => f64::atan,
        "ceil" => f64::ceil,
        "cos" => f64::cos,
        "exp" => f64::exp,
        "floor" => f64::floor,
        "round" => f64::round,
        "sin" => f64::sin,
        "sqrt" => f64::sqrt,
        "tan" => f64::tan
    );

    math.set_function("atan2", atan2, gc_context);
    math.set_function("random", random, gc_context);

    GcCell::allocate(gc_context, math)
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    use crate::avm1::Error;
    use crate::backend::audio::NullAudioBackend;
    use crate::backend::navigator::NullNavigatorBackend;
    use crate::display_object::DisplayObject;
    use crate::movie_clip::MovieClip;
    use gc_arena::rootless_arena;
    use rand::{rngs::SmallRng, SeedableRng};

    macro_rules! test_std {
        ( $test: ident, $name: expr, $($args: expr => $out: expr),* ) => {
            #[test]
            fn $test() -> Result<(), Error> {
                with_avm(19, |avm, context| {
                    let math = create(context.gc_context);
                    let function = math.read().get($name);

                    $(
                        assert_eq!(function.call(avm, context, math, $args)?, $out);
                    )*

                    Ok(())
                })
            }
        };
    }

    fn with_avm<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut Avm1<'gc>, &mut ActionContext<'a, 'gc, '_>) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm = Avm1::new(gc_context, swf_version);
            let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
            let mut context = ActionContext {
                gc_context,
                global_time: 0,
                root,
                start_clip: root,
                active_clip: root,
                target_clip: Some(root),
                target_path: Value::Undefined,
                rng: &mut SmallRng::from_seed([0u8; 16]),
                audio: &mut NullAudioBackend::new(),
                navigator: &mut NullNavigatorBackend::new(),
            };

            test(&mut avm, &mut context)
        })
    }

    test_std!(test_abs, "abs",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(-50.0)] => Value::Number(50.0),
        &[Value::Number(25.0)] => Value::Number(25.0)
    );

    test_std!(test_acos, "acos",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(-1.0)] => Value::Number(f64::acos(-1.0)),
        &[Value::Number(0.0)] => Value::Number(f64::acos(0.0)),
        &[Value::Number(1.0)] => Value::Number(f64::acos(1.0))
    );

    test_std!(test_asin, "asin",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(-1.0)] => Value::Number(f64::asin(-1.0)),
        &[Value::Number(0.0)] => Value::Number(f64::asin(0.0)),
        &[Value::Number(1.0)] => Value::Number(f64::asin(1.0))
    );

    test_std!(test_atan, "atan",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(-1.0)] => Value::Number(f64::atan(-1.0)),
        &[Value::Number(0.0)] => Value::Number(f64::atan(0.0)),
        &[Value::Number(1.0)] => Value::Number(f64::atan(1.0))
    );

    test_std!(test_ceil, "ceil",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(12.5)] => Value::Number(13.0)
    );

    test_std!(test_cos, "cos",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(0.0)] => Value::Number(1.0),
        &[Value::Number(std::f64::consts::PI)] => Value::Number(f64::cos(std::f64::consts::PI))
    );

    test_std!(test_exp, "exp",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(1.0)] => Value::Number(f64::exp(1.0)),
        &[Value::Number(2.0)] => Value::Number(f64::exp(2.0))
    );

    test_std!(test_floor, "floor",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(12.5)] => Value::Number(12.0)
    );

    test_std!(test_round, "round",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(12.5)] => Value::Number(13.0),
        &[Value::Number(23.2)] => Value::Number(23.0)
    );

    test_std!(test_sin, "sin",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(0.0)] => Value::Number(f64::sin(0.0)),
        &[Value::Number(std::f64::consts::PI / 2.0)] => Value::Number(f64::sin(std::f64::consts::PI / 2.0))
    );

    test_std!(test_sqrt, "sqrt",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(0.0)] => Value::Number(f64::sqrt(0.0)),
        &[Value::Number(5.0)] => Value::Number(f64::sqrt(5.0))
    );

    test_std!(test_tan, "tan",
        &[] => Value::Number(NAN),
        &[Value::Null] => Value::Number(NAN),
        &[Value::Number(0.0)] => Value::Number(f64::tan(0.0)),
        &[Value::Number(1.0)] => Value::Number(f64::tan(1.0))
    );

    #[test]
    fn test_atan2_nan() {
        with_avm(19, |avm, context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(atan2(avm, context, *math.read(), &[]), Value::Number(NAN));
            assert_eq!(
                atan2(
                    avm,
                    context,
                    *math.read(),
                    &[Value::Number(1.0), Value::Null]
                ),
                Value::Number(NAN)
            );
            assert_eq!(
                atan2(
                    avm,
                    context,
                    *math.read(),
                    &[Value::Number(1.0), Value::Undefined]
                ),
                Value::Number(NAN)
            );
            assert_eq!(
                atan2(
                    avm,
                    context,
                    *math.read(),
                    &[Value::Undefined, Value::Number(1.0)]
                ),
                Value::Number(NAN)
            );
        });
    }

    #[test]
    fn test_atan2_valid() {
        with_avm(19, |avm, context| {
            let math = GcCell::allocate(context.gc_context, create(context.gc_context));
            assert_eq!(
                atan2(avm, context, *math.read(), &[Value::Number(10.0)]),
                Value::Number(std::f64::consts::FRAC_PI_2)
            );
            assert_eq!(
                atan2(
                    avm,
                    context,
                    *math.read(),
                    &[Value::Number(1.0), Value::Number(2.0)]
                ),
                Value::Number(0.4636476090008061)
            );
        });
    }
}
