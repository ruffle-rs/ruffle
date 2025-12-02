//! `Math` impl

use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_1075, make_error_1076};
use crate::avm2::object::Object;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ClassObject, Error};
use num_traits::ToPrimitive;

macro_rules! wrap_std {
    ($name:ident, $std:expr) => {
        pub fn $name<'gc>(
            _activation: &mut Activation<'_, 'gc>,
            _this: Value<'gc>,
            args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            let input = args.get_f64(0);
            Ok($std(input).into())
        }
    };
}

wrap_std!(abs, f64::abs);
wrap_std!(acos, f64::acos);
wrap_std!(asin, f64::asin);
wrap_std!(atan, f64::atan);
wrap_std!(ceil, f64::ceil);
wrap_std!(cos, f64::cos);
wrap_std!(exp, f64::exp);
wrap_std!(floor, f64::floor);
wrap_std!(log, f64::ln);
wrap_std!(sin, f64::sin);
wrap_std!(sqrt, f64::sqrt);
wrap_std!(tan, f64::tan);

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err(make_error_1075(activation))
}

pub fn math_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Err(make_error_1076(activation))
}

pub fn round<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args.get_f64(0);

    // Note that Flash Math.round always rounds toward infinity,
    // unlike Rust f32::round which rounds away from zero.
    let ret = (x + 0.5).floor();
    Ok(ret.into())
}

pub fn atan2<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let y = args.get_f64(0);
    let x = args.get_f64(1);

    Ok(f64::atan2(y, x).into())
}

pub fn max<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut cur_max = f64::NEG_INFINITY;
    for arg in args {
        let val = arg.coerce_to_number(activation)?;
        if val.is_nan() {
            return Ok(f64::NAN.into());
        } else if val.total_cmp(&cur_max).is_gt() {
            cur_max = val;
        };
    }
    Ok(cur_max.into())
}

pub fn min<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut cur_min = f64::INFINITY;
    for arg in args {
        let val = arg.coerce_to_number(activation)?;
        if val.is_nan() {
            return Ok(f64::NAN.into());
        } else if val.total_cmp(&cur_min).is_lt() {
            cur_min = val;
        }
    }
    Ok(cur_min.into())
}

pub fn pow<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let n = args.get_f64(0);
    let p = args.get_f64(1);

    // This condition is the simplest one that covers all special cases,
    // so use it in order to create a fast path for finite n and p, which is
    // the most common configuration.
    if !n.is_finite() || !p.is_finite() {
        match (n, p) {
            // Special case: If p is NaN, the result is NaN.
            (_, _) if p.is_nan() => return Ok(f64::NAN.into()),
            // Special case: If p is ±Infinity and n is ±1, the result is NaN.
            (1.0, _) | (-1.0, _) => {
                // If (1) n or p is not finite, (2) p is not NaN, (3) n is finite,
                // p has to be infinite.
                debug_assert!(p.is_infinite());
                return Ok(f64::NAN.into());
            }
            // Special case: If n is -Infinity and p < 0 and p is a negative even integer, Flash Player returns -0.
            (f64::NEG_INFINITY, _) if p.to_i64().is_some_and(|i| i % 2 == 0 && i < 0) => {
                return Ok(Value::Number(-0.0))
            }
            _ => {
                // Fall back to regular powf
            }
        }
    }

    Ok(f64::powf(n, p).into())
}

pub fn random<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/MathUtils.cpp#L1731C24-L1731C44
    // This generated a restricted set of 'f64' values, which some SWFs implicitly rely on.
    const MAX_VAL: u32 = 0x7FFFFFFF;
    let rand = activation.context.rng.generate_random_number();
    Ok(((rand as f64) / (MAX_VAL as f64 + 1f64)).into())
}
