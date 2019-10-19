use crate::avm1::object::Attribute::*;
use crate::avm1::{Avm1, Object, UpdateContext, Value};
use gc_arena::{GcCell, MutationContext};
use rand::Rng;
use std::f64::NAN;

macro_rules! wrap_std {
    ( $object: ident, $gc_context: ident, $($name:expr => $std:path),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |_avm, _context, _this, args| -> Value<'gc> {
                    if let Some(input) = args.get(0) {
                        Value::Number($std(input.as_number()))
                    } else {
                        Value::Number(NAN)
                    }
                },
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
            );
        )*
    }};
}

fn atan2<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
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
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    _args: &[Value<'gc>],
) -> Value<'gc> {
    Value::Number(action_context.rng.gen_range(0.0f64, 1.0f64))
}

pub fn create<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Object<'gc>> {
    let mut math = Object::object(gc_context);

    math.force_set(
        "E",
        Value::Number(std::f64::consts::E),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "LN10",
        Value::Number(std::f64::consts::LN_10),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "LN2",
        Value::Number(std::f64::consts::LN_2),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "LOG10E",
        Value::Number(std::f64::consts::LOG10_E),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "LOG2E",
        Value::Number(std::f64::consts::LOG2_E),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "PI",
        Value::Number(std::f64::consts::PI),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "SQRT1_2",
        Value::Number(std::f64::consts::FRAC_1_SQRT_2),
        DontDelete | ReadOnly | DontEnum,
    );
    math.force_set(
        "SQRT2",
        Value::Number(std::f64::consts::SQRT_2),
        DontDelete | ReadOnly | DontEnum,
    );

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

    math.force_set_function("atan2", atan2, gc_context, DontDelete | ReadOnly | DontEnum);
    math.force_set_function(
        "random",
        random,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
    );

    GcCell::allocate(gc_context, math)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::Error;

    macro_rules! test_std {
        ( $test: ident, $name: expr, $([$($arg: expr),*] => $out: expr),* ) => {
            #[test]
            fn $test() -> Result<(), Error> {
                with_avm(19, |avm, context, _root| {
                    let math = create(context.gc_context);
                    let function = math.read().get($name, avm, context, math).unwrap();

                    $(
                        #[allow(unused_mut)]
                        let mut args: Vec<Value> = Vec::new();
                        $(
                            args.push($arg.into());
                        )*
                        assert_eq!(function.call(avm, context, math, &args)?, Some($out.into()));
                    )*

                    Ok(())
                })
            }
        };
    }

    test_std!(test_abs, "abs",
        [] => NAN,
        [Value::Null] => NAN,
        [-50.0] => 50.0,
        [25.0] => 25.0
    );

    test_std!(test_acos, "acos",
        [] => NAN,
        [Value::Null] => NAN,
        [-1.0] => f64::acos(-1.0),
        [0.0] => f64::acos(0.0),
        [1.0] => f64::acos(1.0)
    );

    test_std!(test_asin, "asin",
        [] => NAN,
        [Value::Null] => NAN,
        [-1.0] => f64::asin(-1.0),
        [0.0] => f64::asin(0.0),
        [1.0] => f64::asin(1.0)
    );

    test_std!(test_atan, "atan",
        [] => NAN,
        [Value::Null] => NAN,
        [-1.0] => f64::atan(-1.0),
        [0.0] => f64::atan(0.0),
        [1.0] => f64::atan(1.0)
    );

    test_std!(test_ceil, "ceil",
        [] => NAN,
        [Value::Null] => NAN,
        [12.5] => 13.0
    );

    test_std!(test_cos, "cos",
        [] => NAN,
        [Value::Null] => NAN,
        [0.0] => 1.0,
        [std::f64::consts::PI] => f64::cos(std::f64::consts::PI)
    );

    test_std!(test_exp, "exp",
        [] => NAN,
        [Value::Null] => NAN,
        [1.0] => f64::exp(1.0),
        [2.0] => f64::exp(2.0)
    );

    test_std!(test_floor, "floor",
        [] => NAN,
        [Value::Null] => NAN,
        [12.5] => 12.0
    );

    test_std!(test_round, "round",
        [] => NAN,
        [Value::Null] => NAN,
        [12.5] => 13.0,
        [23.2] => 23.0
    );

    test_std!(test_sin, "sin",
        [] => NAN,
        [Value::Null] => NAN,
        [0.0] => f64::sin(0.0),
        [std::f64::consts::PI / 2.0] => f64::sin(std::f64::consts::PI / 2.0)
    );

    test_std!(test_sqrt, "sqrt",
        [] => NAN,
        [Value::Null] => NAN,
        [0.0] => f64::sqrt(0.0),
        [5.0] => f64::sqrt(5.0)
    );

    test_std!(test_tan, "tan",
        [] => NAN,
        [Value::Null] => NAN,
        [0.0] => f64::tan(0.0),
        [1.0] => f64::tan(1.0)
    );

    #[test]
    fn test_atan2_nan() {
        with_avm(19, |avm, context, _root| {
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
        with_avm(19, |avm, context, _root| {
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
                Value::Number(f64::atan2(1.0, 2.0))
            );
        });
    }
}
