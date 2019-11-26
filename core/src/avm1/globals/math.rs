use crate::avm1::object::ObjectCell;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, UpdateContext, Value};
use gc_arena::{GcCell, MutationContext};
use rand::Rng;
use std::f64::NAN;

macro_rules! wrap_std {
    ( $object: ident, $gc_context: ident, $proto: ident, $($name:expr => $std:path),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |avm, context, _this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(input) = args.get(0) {
                        Ok($std(input.as_number(avm, context)?).into())
                    } else {
                        Ok(NAN.into())
                    }
                },
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $proto
            );
        )*
    }};
}

fn atan2<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: ObjectCell<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            return Ok(y
                .as_number(avm, context)?
                .atan2(x.as_number(avm, context)?)
                .into());
        } else {
            return Ok(y.as_number(avm, context)?.atan2(0.0).into());
        }
    }
    Ok(NAN.into())
}

pub fn random<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: ObjectCell<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(action_context.rng.gen_range(0.0f64, 1.0f64).into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<ObjectCell<'gc>>,
    fn_proto: Option<ObjectCell<'gc>>,
) -> ObjectCell<'gc> {
    let mut math = ScriptObject::object(gc_context, proto);

    math.define_value(
        "E",
        std::f64::consts::E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "LN10",
        std::f64::consts::LN_10.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "LN2",
        std::f64::consts::LN_2.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "LOG10E",
        std::f64::consts::LOG10_E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "LOG2E",
        std::f64::consts::LOG2_E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "PI",
        std::f64::consts::PI.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "SQRT1_2",
        std::f64::consts::FRAC_1_SQRT_2.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        "SQRT2",
        std::f64::consts::SQRT_2.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    wrap_std!(math, gc_context, fn_proto,
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

    math.force_set_function(
        "atan2",
        atan2,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );
    math.force_set_function(
        "random",
        random,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    GcCell::allocate(gc_context, Box::new(math))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::Error;

    macro_rules! test_std {
        ( $test: ident, $name: expr, $($versions: expr => { $([$($arg: expr),*] => $out: expr),* }),* ) => {
            #[test]
            fn $test() -> Result<(), Error> {
                $(
                    for version in &$versions {
                        let _ = with_avm(*version, |avm, context, _root| -> Result<(), Error> {
                            let math = create(context.gc_context, Some(avm.prototypes().object), Some(avm.prototypes().function));
                            let function = math.read().get($name, avm, context, math)?.unwrap_immediate();

                            $(
                                #[allow(unused_mut)]
                                let mut args: Vec<Value> = Vec::new();
                                $(
                                    args.push($arg.into());
                                )*
                                assert_eq!(function.call(avm, context, math, &args)?, ReturnValue::Immediate($out.into()), "{:?} => {:?} in swf {}", args, $out, version);
                            )*

                            Ok(())
                        })?;
                    }
                )*

                Ok(())
            }
        };
    }

    test_std!(test_abs, "abs",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-50.0] => 50.0,
            [25.0] => 25.0
        }
    );

    test_std!(test_acos, "acos",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::acos(-1.0),
            [0.0] => f64::acos(0.0),
            [1.0] => f64::acos(1.0)
        }
    );

    test_std!(test_asin, "asin",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::asin(-1.0),
            [0.0] => f64::asin(0.0),
            [1.0] => f64::asin(1.0)
        }
    );

    test_std!(test_atan, "atan",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::atan(-1.0),
            [0.0] => f64::atan(0.0),
            [1.0] => f64::atan(1.0)
        }
    );

    test_std!(test_ceil, "ceil",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [12.5] => 13.0
        }
    );

    test_std!(test_cos, "cos",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => 1.0,
            [std::f64::consts::PI] => f64::cos(std::f64::consts::PI)
        }
    );

    test_std!(test_exp, "exp",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [1.0] => f64::exp(1.0),
            [2.0] => f64::exp(2.0)
        }
    );

    test_std!(test_floor, "floor",
        [19] => {
            [] => NAN,
            [Value::Undefined] => NAN,
            [Value::Null] => NAN,
            [Value::Bool(false)] => 0.0,
            [Value::Bool(true)] => 1.0,
            [12.5] => 12.0
        },
        [6] => {
            [] => NAN,
            [Value::Undefined] => 0.0,
            [Value::Null] => 0.0,
            [Value::Bool(false)] => 0.0,
            [Value::Bool(true)] => 1.0,
            [12.5] => 12.0
        }
    );

    test_std!(test_round, "round",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [12.5] => 13.0,
            [23.2] => 23.0
        }
    );

    test_std!(test_sin, "sin",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::sin(0.0),
            [std::f64::consts::PI / 2.0] => f64::sin(std::f64::consts::PI / 2.0)
        }
    );

    test_std!(test_sqrt, "sqrt",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::sqrt(0.0),
            [5.0] => f64::sqrt(5.0)
        }
    );

    test_std!(test_tan, "tan",
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::tan(0.0),
            [1.0] => f64::tan(1.0)
        }
    );

    #[test]
    fn test_atan2_nan() {
        with_avm(19, |avm, context, _root| {
            let math = GcCell::allocate(
                context.gc_context,
                create(
                    context.gc_context,
                    Some(avm.prototypes().object),
                    Some(avm.prototypes().function),
                ),
            );

            assert_eq!(atan2(avm, context, *math.read(), &[]).unwrap(), NAN.into());
            assert_eq!(
                atan2(avm, context, *math.read(), &[1.0.into(), Value::Null]).unwrap(),
                NAN.into()
            );
            assert_eq!(
                atan2(avm, context, *math.read(), &[1.0.into(), Value::Undefined]).unwrap(),
                NAN.into()
            );
            assert_eq!(
                atan2(avm, context, *math.read(), &[Value::Undefined, 1.0.into()]).unwrap(),
                NAN.into()
            );
        });
    }

    #[test]
    fn test_atan2_valid() {
        with_avm(19, |avm, context, _root| {
            let math = GcCell::allocate(
                context.gc_context,
                create(
                    context.gc_context,
                    Some(avm.prototypes().object),
                    Some(avm.prototypes().function),
                ),
            );

            assert_eq!(
                atan2(avm, context, *math.read(), &[10.0.into()]).unwrap(),
                std::f64::consts::FRAC_PI_2.into()
            );
            assert_eq!(
                atan2(avm, context, *math.read(), &[1.0.into(), 2.0.into()]).unwrap(),
                f64::atan2(1.0, 2.0).into()
            );
        });
    }
}
