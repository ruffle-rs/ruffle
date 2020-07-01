use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute::*;
use crate::avm1::{ScriptObject, TObject, UpdateContext, Value};
use gc_arena::MutationContext;
use rand::Rng;
use std::f64::{INFINITY, NAN, NEG_INFINITY};

macro_rules! wrap_std {
    ( $object: ident, $gc_context: ident, $proto: ident, $($name:expr => $std:path),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |activation, context, _this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(input) = args.get(0) {
                        Ok($std(input.coerce_to_f64(activation, context)?).into())
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
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            return Ok(y
                .coerce_to_f64(activation, context)?
                .atan2(x.coerce_to_f64(activation, context)?)
                .into());
        } else {
            return Ok(y.coerce_to_f64(activation, context)?.atan2(0.0).into());
        }
    }
    Ok(NAN.into())
}

fn pow<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            let x = x.coerce_to_f64(activation, context)?;
            if x.is_nan() {
                return Ok(NAN.into());
            }
            return Ok(y.coerce_to_f64(activation, context)?.powf(x).into());
        }
    }
    Ok(NAN.into())
}

fn round<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(x) = args.get(0) {
        let x = x.coerce_to_f64(activation, context)?;
        // Note that Flash Math.round always rounds toward infinity,
        // unlike Rust f32::round which rounds away from zero.
        let ret = (x + 0.5).floor();
        return Ok(ret.into());
    }
    Ok(NAN.into())
}

fn max<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(a) = args.get(0) {
        return if let Some(b) = args.get(1) {
            match a.abstract_lt(b.to_owned(), activation, context)? {
                Value::Bool(value) => {
                    if value {
                        Ok(b.coerce_to_f64(activation, context)?.into())
                    } else {
                        Ok(a.coerce_to_f64(activation, context)?.into())
                    }
                }
                _ => Ok(NAN.into()),
            }
        } else {
            Ok(NAN.into())
        };
    }
    Ok(NEG_INFINITY.into())
}

fn min<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(a) = args.get(0) {
        return if let Some(b) = args.get(1) {
            match a.abstract_lt(b.to_owned(), activation, context)? {
                Value::Bool(value) => {
                    if value {
                        Ok(a.coerce_to_f64(activation, context)?.into())
                    } else {
                        Ok(b.coerce_to_f64(activation, context)?.into())
                    }
                }
                _ => Ok(NAN.into()),
            }
        } else {
            Ok(NAN.into())
        };
    }
    Ok(INFINITY.into())
}

pub fn random<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(action_context.rng.gen_range(0.0f64, 1.0f64).into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut math = ScriptObject::object(gc_context, proto);

    math.define_value(
        gc_context,
        "E",
        std::f64::consts::E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "LN10",
        std::f64::consts::LN_10.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "LN2",
        std::f64::consts::LN_2.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "LOG10E",
        std::f64::consts::LOG10_E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "LOG2E",
        std::f64::consts::LOG2_E.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "PI",
        std::f64::consts::PI.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
        "SQRT1_2",
        std::f64::consts::FRAC_1_SQRT_2.into(),
        DontDelete | ReadOnly | DontEnum,
    );
    math.define_value(
        gc_context,
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
        "sin" => f64::sin,
        "sqrt" => f64::sqrt,
        "tan" => f64::tan,
        "log" => f64::ln
    );

    math.force_set_function(
        "atan2",
        atan2,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );
    math.force_set_function(
        "pow",
        pow,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );
    math.force_set_function(
        "max",
        max,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );
    math.force_set_function(
        "min",
        min,
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
    math.force_set_function(
        "round",
        round,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    math.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::test_utils::with_avm;

    fn setup<'gc>(
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Object<'gc> {
        create(
            context.gc_context,
            Some(activation.avm().prototypes().object),
            Some(activation.avm().prototypes().function),
        )
    }

    test_method!(test_abs, "abs", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-50.0] => 50.0,
            [25.0] => 25.0
        }
    );

    test_method!(test_acos, "acos", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::acos(-1.0),
            [0.0] => f64::acos(0.0),
            [1.0] => f64::acos(1.0)
        }
    );

    test_method!(test_asin, "asin", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::asin(-1.0),
            [0.0] => f64::asin(0.0),
            [1.0] => f64::asin(1.0)
        }
    );

    test_method!(test_atan, "atan", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [-1.0] => f64::atan(-1.0),
            [0.0] => f64::atan(0.0),
            [1.0] => f64::atan(1.0)
        }
    );

    test_method!(test_ceil, "ceil", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [12.5] => 13.0
        }
    );

    test_method!(test_cos, "cos", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => 1.0,
            [std::f64::consts::PI] => f64::cos(std::f64::consts::PI)
        }
    );

    test_method!(test_exp, "exp", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [1.0] => f64::exp(1.0),
            [2.0] => f64::exp(2.0)
        }
    );

    test_method!(test_floor, "floor", setup,
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

    test_method!(test_round, "round", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [Value::Undefined] => NAN,
            [12.5] => 13.0,
            [23.2] => 23.0,
            [23.5] => 24.0,
            [23.7] => 24.0,
            [-23.2] => -23.0,
            [-23.5] => -23.0,
            [-23.7] => -24.0,
            [std::f64::NAN] => std::f64::NAN,
            [std::f64::INFINITY] => std::f64::INFINITY,
            [std::f64::NEG_INFINITY] => std::f64::NEG_INFINITY
        },
        [5, 6] => {
            [] => NAN,
            [Value::Null] => 0.0,
            [Value::Undefined] => 0.0,
            [std::f64::NAN] => std::f64::NAN
        }
    );

    test_method!(test_sin, "sin", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::sin(0.0),
            [std::f64::consts::PI / 2.0] => f64::sin(std::f64::consts::PI / 2.0)
        }
    );

    test_method!(test_sqrt, "sqrt", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::sqrt(0.0),
            [5.0] => f64::sqrt(5.0)
        }
    );

    test_method!(test_tan, "tan", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [0.0] => f64::tan(0.0),
            [1.0] => f64::tan(1.0)
        }
    );

    test_method!(test_pow, "pow", setup,
        [5, 6, 7, 8] => {
            [] => NAN,
            [1.0] => NAN,
            [NAN] => NAN,
            [Value::Null] => NAN,
            [Value::Undefined] => NAN,
            ["5"] => NAN,
            [1.0, 2.0] => 1.0,
            [3.0, 2.0, 1.0] => 9.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 1.0,
            [Value::Undefined, 3.0] => 0.0
        },
        [7, 8] => {
            [1.0, Value::Null] => NAN,
            [Value::Undefined, 3.0] => NAN
        }
    );

    test_method!(test_log, "log", setup,
        [19] => {
            [] => NAN,
            [Value::Null] => NAN,
            [2.0] => f64::ln(2.0),
            [0.0] => f64::ln(0.0),
            [1.0] => f64::ln(1.0)
        }
    );

    test_method!(test_max, "max", setup,
        [5, 6, 7, 8] => {
            [] => NEG_INFINITY,
            [1.0] => NAN,
            [NAN] => NAN,
            [Value::Null] => NAN,
            [Value::Undefined] => NAN,
            ["5"] => NAN,
            [1.0, 2.0] => 2.0,
            [3.0, 2.0, 1.0] => 3.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 1.0,
            [Value::Undefined, 3.0] => 3.0
        },
        [7, 8] => {
            [1.0, Value::Null] => NAN,
            [Value::Undefined, 3.0] => NAN
        }
    );

    test_method!(test_min, "min", setup,
        [5, 6, 7, 8] => {
            [] => INFINITY,
            [1.0] => NAN,
            [NAN] => NAN,
            [Value::Null] => NAN,
            [Value::Undefined] => NAN,
            ["5"] => NAN,
            [1.0, 2.0] => 1.0,
            [3.0, 2.0, 1.0] => 2.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 0.0,
            [Value::Undefined, 3.0] => 0.0
        },
        [7, 8] => {
            [1.0, Value::Null] => NAN,
            [Value::Undefined, 3.0] => NAN
        }
    );

    #[test]
    fn test_atan2_nan() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let math = create(
                context.gc_context,
                Some(activation.avm().prototypes().object),
                Some(activation.avm().prototypes().function),
            );

            assert_eq!(atan2(activation, context, math, &[]).unwrap(), NAN.into());
            assert_eq!(
                atan2(activation, context, math, &[1.0.into(), Value::Null]).unwrap(),
                NAN.into()
            );
            assert_eq!(
                atan2(activation, context, math, &[1.0.into(), Value::Undefined]).unwrap(),
                NAN.into()
            );
            assert_eq!(
                atan2(activation, context, math, &[Value::Undefined, 1.0.into()]).unwrap(),
                NAN.into()
            );
            Ok(())
        });
    }

    #[test]
    fn test_atan2_valid() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let math = create(
                context.gc_context,
                Some(activation.avm().prototypes().object),
                Some(activation.avm().prototypes().function),
            );

            assert_eq!(
                atan2(activation, context, math, &[10.0.into()]).unwrap(),
                std::f64::consts::FRAC_PI_2.into()
            );
            assert_eq!(
                atan2(activation, context, math, &[1.0.into(), 2.0.into()]).unwrap(),
                f64::atan2(1.0, 2.0).into()
            );
            Ok(())
        });
    }
}
