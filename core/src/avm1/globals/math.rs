use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::Object;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ScriptObject, Value};
use crate::string::StringContext;

use rand::Rng;
use std::f64::consts;

macro_rules! wrap_std {
    ($std:path) => {
        |activation, _this, args| {
            if let Some(input) = args.get(0) {
                Ok($std(input.coerce_to_f64(activation)?).into())
            } else {
                Ok(f64::NAN.into())
            }
        }
    };
}

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "E" => float(consts::E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN10" => float(consts::LN_10; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN2" => float(consts::LN_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG10E" => float(consts::LOG10_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG2E" => float(consts::LOG2_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PI" => float(consts::PI; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SQRT1_2" => float(consts::FRAC_1_SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SQRT2" => float(consts::SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "abs" => method(wrap_std!(f64::abs); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "acos" => method(wrap_std!(f64::acos); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "asin" => method(wrap_std!(f64::asin); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan" => method(wrap_std!(f64::atan); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ceil" => method(wrap_std!(f64::ceil); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "cos" => method(wrap_std!(f64::cos); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "exp" => method(wrap_std!(f64::exp); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "floor" => method(wrap_std!(f64::floor); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sin" => method(wrap_std!(f64::sin); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sqrt" => method(wrap_std!(f64::sqrt); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "tan" => method(wrap_std!(f64::tan); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "log" => method(wrap_std!(f64::ln); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan2" => method(atan2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "pow" => method(pow; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "max" => method(max; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "min" => method(min; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "random" => method(random; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "round" => method(round; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

fn atan2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            return Ok(y
                .coerce_to_f64(activation)?
                .atan2(x.coerce_to_f64(activation)?)
                .into());
        } else {
            return Ok(y.coerce_to_f64(activation)?.atan2(0.0).into());
        }
    }
    Ok(f64::NAN.into())
}

fn pow<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(y) = args.get(0) {
        if let Some(x) = args.get(1) {
            let x = x.coerce_to_f64(activation)?;
            if x.is_nan() {
                return Ok(f64::NAN.into());
            }
            return Ok(y.coerce_to_f64(activation)?.powf(x).into());
        }
    }
    Ok(f64::NAN.into())
}

fn round<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(x) = args.get(0) {
        let x = x.coerce_to_f64(activation)?;
        // Note that Flash Math.round always rounds toward infinity,
        // unlike Rust f32::round which rounds away from zero.
        let ret = (x + 0.5).floor();
        return Ok(ret.into());
    }
    Ok(f64::NAN.into())
}

fn max<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let result = if let Some(a) = args.get(0) {
        let a = a.coerce_to_f64(activation)?;
        if let Some(b) = args.get(1) {
            let b = b.coerce_to_f64(activation)?;
            use std::cmp::Ordering;
            match a.partial_cmp(&b) {
                Some(Ordering::Less) => b,
                Some(Ordering::Equal) => a,
                Some(Ordering::Greater) => a,
                None => f64::NAN,
            }
        } else {
            f64::NAN
        }
    } else {
        f64::NEG_INFINITY
    };
    Ok(result.into())
}

fn min<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let result = if let Some(a) = args.get(0) {
        let a = a.coerce_to_f64(activation)?;
        if let Some(b) = args.get(1) {
            let b = b.coerce_to_f64(activation)?;
            use std::cmp::Ordering;
            match a.partial_cmp(&b) {
                Some(Ordering::Less) => a,
                Some(Ordering::Equal) => a,
                Some(Ordering::Greater) => b,
                None => f64::NAN,
            }
        } else {
            f64::NAN
        }
    } else {
        f64::INFINITY
    };
    Ok(result.into())
}

pub fn random<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/MathUtils.cpp#L1731C24-L1731C44
    // This generated a restricted set of 'f64' values, which some SWFs implicitly rely on.
    const MAX_VAL: u32 = 0x7FFFFFFF;
    let rand = activation.context.rng.gen_range(0..MAX_VAL);
    Ok(((rand as f64) / (MAX_VAL as f64 + 1f64)).into())
}

pub fn create<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let math = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, math, fn_proto);
    math.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::test_utils::with_avm;

    fn setup<'gc>(activation: &mut Activation<'_, 'gc>) -> Object<'gc> {
        let object_proto = activation.context.avm1.prototypes().object;
        let function_proto = activation.context.avm1.prototypes().function;
        create(activation.strings(), object_proto, function_proto)
    }

    test_method!(test_abs, "abs", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [-50.0] => 50.0,
            [25.0] => 25.0
        }
    );

    test_method!(test_acos, "acos", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [-1.0] => f64::acos(-1.0),
            [0.0] => f64::acos(0.0),
            [1.0] => 0.0 // f64::acos(1.0)
        }
    );

    test_method!(test_asin, "asin", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [-1.0] => f64::asin(-1.0),
            [0.0] => 0.0, // f64::asin(0.0),
            [1.0] => f64::asin(1.0)
        }
    );

    test_method!(test_atan, "atan", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [-1.0] => f64::atan(-1.0),
            [0.0] => 0.0, // f64::atan(0.0),
            [1.0] => f64::atan(1.0)
        }
    );

    test_method!(test_ceil, "ceil", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [12.5] => 13.0
        }
    );

    test_method!(test_cos, "cos", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [0.0] => 1.0,
            [std::f64::consts::PI] => -1.0 // f64::cos(std::f64::consts::PI)
        }
    );

    test_method!(test_exp, "exp", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            @epsilon(1e-12) [1.0] => f64::from_bits(0x4005bf0a8b145769), // f64::exp(1.0), e, 2.718281828459045
            @epsilon(1e-12) [2.0] => f64::from_bits(0x401d8e64b8d4ddae)  // f64::exp(2.0), e^2, 7.3890560989306495
        }
    );

    test_method!(test_floor, "floor", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Undefined] => f64::NAN,
            [Value::Null] => f64::NAN,
            [Value::Bool(false)] => 0.0,
            [Value::Bool(true)] => 1.0,
            [12.5] => 12.0
        },
        [6] => {
            [] => f64::NAN,
            [Value::Undefined] => 0.0,
            [Value::Null] => 0.0,
            [Value::Bool(false)] => 0.0,
            [Value::Bool(true)] => 1.0,
            [12.5] => 12.0
        }
    );

    test_method!(test_round, "round", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [Value::Undefined] => f64::NAN,
            [12.5] => 13.0,
            [23.2] => 23.0,
            [23.5] => 24.0,
            [23.7] => 24.0,
            [-23.2] => -23.0,
            [-23.5] => -23.0,
            [-23.7] => -24.0,
            [f64::NAN] => f64::NAN,
            [f64::INFINITY] => f64::INFINITY,
            [f64::NEG_INFINITY] => f64::NEG_INFINITY
        },
        [5, 6] => {
            [] => f64::NAN,
            [Value::Null] => 0.0,
            [Value::Undefined] => 0.0,
            [f64::NAN] => f64::NAN
        }
    );

    test_method!(test_sin, "sin", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [0.0] => 0.0, // f64::sin(0.0),
            [std::f64::consts::PI / 2.0] => 1.0 // f64::sin(std::f64::consts::PI / 2.0)
        }
    );

    test_method!(test_sqrt, "sqrt", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [0.0] => 0.0, // f64::sqrt(0.0),
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [5.0] => f64::sqrt(5.0)
        }
    );

    test_method!(test_tan, "tan", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            [0.0] => 0.0, // f64::tan(0.0),
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [1.0] => f64::tan(1.0)
        }
    );

    test_method!(test_pow, "pow", setup,
        [5, 6, 7, 8] => {
            [] => f64::NAN,
            [1.0] => f64::NAN,
            [f64::NAN] => f64::NAN,
            [Value::Null] => f64::NAN,
            [Value::Undefined] => f64::NAN,
            ["5"] => f64::NAN,
            [1.0, 2.0] => 1.0,
            [3.0, 2.0, 1.0] => 9.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 1.0,
            [Value::Undefined, 3.0] => 0.0
        },
        [7, 8] => {
            [1.0, Value::Null] => f64::NAN,
            [Value::Undefined, 3.0] => f64::NAN
        }
    );

    test_method!(test_log, "log", setup,
        [19] => {
            [] => f64::NAN,
            [Value::Null] => f64::NAN,
            // TODO: figure out the exact f64 returned, and add @epsilon as needed, see test_exp
            [2.0] => f64::ln(2.0),
            [0.0] => f64::NEG_INFINITY, // f64::ln(0.0),
            [1.0] => 0 // f64::ln(1.0)
        }
    );

    test_method!(test_max, "max", setup,
        [5, 6, 7, 8] => {
            [] => f64::NEG_INFINITY,
            [1.0] => f64::NAN,
            [f64::NAN] => f64::NAN,
            [Value::Null] => f64::NAN,
            [Value::Undefined] => f64::NAN,
            ["5"] => f64::NAN,
            [1.0, 2.0] => 2.0,
            [3.0, 2.0, 1.0] => 3.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 1.0,
            [Value::Undefined, 3.0] => 3.0
        },
        [7, 8] => {
            [1.0, Value::Null] => f64::NAN,
            [Value::Undefined, 3.0] => f64::NAN
        }
    );

    test_method!(test_min, "min", setup,
        [5, 6, 7, 8] => {
            [] => f64::INFINITY,
            [1.0] => f64::NAN,
            [f64::NAN] => f64::NAN,
            [Value::Null] => f64::NAN,
            [Value::Undefined] => f64::NAN,
            ["5"] => f64::NAN,
            [1.0, 2.0] => 1.0,
            [3.0, 2.0, 1.0] => 2.0
        },
        [5, 6] => {
            [1.0, Value::Null] => 0.0,
            [Value::Undefined, 3.0] => 0.0
        },
        [7, 8] => {
            [1.0, Value::Null] => f64::NAN,
            [Value::Undefined, 3.0] => f64::NAN
        }
    );

    #[test]
    fn test_atan2_nan() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            let math = setup(activation);

            assert_eq!(atan2(activation, math, &[]).unwrap(), f64::NAN.into());
            assert_eq!(
                atan2(activation, math, &[1.into(), Value::Null]).unwrap(),
                f64::NAN.into()
            );
            assert_eq!(
                atan2(activation, math, &[1.into(), Value::Undefined]).unwrap(),
                f64::NAN.into()
            );
            assert_eq!(
                atan2(activation, math, &[Value::Undefined, 1.into()]).unwrap(),
                f64::NAN.into()
            );
            Ok(())
        });
    }

    #[test]
    fn test_atan2_valid() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            let math = setup(activation);

            assert_eq!(
                atan2(activation, math, &[10.into()]).unwrap(),
                std::f64::consts::FRAC_PI_2.into()
            );
            assert_eq!(
                atan2(activation, math, &[1.into(), 2.into()]).unwrap(),
                f64::atan2(1.0, 2.0).into()
            );
            Ok(())
        });
    }
}
