//! `Math` impl

use crate::avm2::activation::Activation;
use crate::avm2::error::type_error;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::{ClassObject, Error};
use num_traits::ToPrimitive;
use rand::Rng;

macro_rules! wrap_std {
    ($name:ident, $std:expr) => {
        pub fn $name<'gc>(
            _activation: &mut Activation<'_, 'gc>,
            _this: Value<'gc>,
            args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            let input = args[0].as_f64();
            Ok($std(input).into())
        }
    };
}

wrap_std!(acos, f64::acos);
wrap_std!(asin, f64::asin);
wrap_std!(atan, f64::atan);
wrap_std!(cos, f64::cos);
wrap_std!(exp, f64::exp);
wrap_std!(log, f64::ln);
wrap_std!(sin, f64::sin);
wrap_std!(sqrt, f64::sqrt);
wrap_std!(tan, f64::tan);

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err(Error::avm_error(type_error(
        activation,
        "Error #1075: Math is not a function.",
        1075,
    )?))
}

pub fn math_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Err(Error::avm_error(type_error(
        activation,
        "Error #1076: Math is not a constructor.",
        1076,
    )?))
}

pub fn round<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args[0].as_f64();

    // Note that Flash Math.round always rounds toward infinity,
    // unlike Rust f32::round which rounds away from zero.
    let ret = (x + 0.5).floor();
    match ret.to_i32() {
        Some(num) => Ok(Value::Integer(num)),
        None => Ok(Value::Number(ret)),
    }
}

pub fn atan2<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let y = args[0].as_f64();
    let x = args[1].as_f64();

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
        } else if val > cur_max
            || (val == cur_max && !val.is_sign_negative() && cur_max.is_sign_negative())
        {
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
        } else if val < cur_min
            || (val == cur_min && val.is_sign_negative() && !cur_min.is_sign_negative())
        {
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
    let n = args[0].as_f64();
    let p = args[1].as_f64();
    match (n, p) {
        (_, _) if p.is_nan() => Ok(f64::NAN.into()),
        // Special case: If p is ±Infinity and n is ±1, the result is NaN.
        (1.0, _) | (-1.0, _) if p.is_infinite() => Ok(f64::NAN.into()),
        // Special case: If n is -Infinity and p < 0 and p is a negative even integer, Flash Player returns -0.
        (f64::NEG_INFINITY, _) if p.to_i64().is_some_and(|i| i % 2 == 0 && i < 0) => {
            Ok(Value::Number(-0.0))
        }
        (_, _) => Ok(f64::powf(n, p).into()),
    }
}

pub fn random<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/MathUtils.cpp#L1731C24-L1731C44
    // This generated a restricted set of 'f64' values, which some SWFs implicitly rely on.
    const MAX_VAL: u32 = 0x7FFFFFFF;
    let rand = activation.context.rng.random_range(0..MAX_VAL);
    Ok(((rand as f64) / (MAX_VAL as f64 + 1f64)).into())
}

pub fn abs<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args[0];
    match input {
        Value::Integer(i) => Ok(i.abs().into()),
        Value::Number(-0.0) => Ok(Value::Integer(0)),
        _ => Ok(f64::abs(input.as_f64()).into()),
    }
}

pub fn ceil<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args[0].as_f64();

    if input.is_nan() {
        Ok(Value::Number(f64::NAN))
    } else if input.is_infinite() {
        Ok(Value::Number(input))
    } else {
        let ceiled = input.ceil();
        // Special case: if input was negative and ceil result is 0, preserve -0.0
        if ceiled == 0.0 && input.is_sign_negative() {
            Ok(Value::Number(-0.0))
        } else if ceiled >= i32::MIN as f64 && ceiled <= i32::MAX as f64 {
            Ok(Value::Integer(ceiled as i32))
        } else {
            Ok(Value::Number(ceiled))
        }
    }
}

pub fn floor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args[0];
    let num = input.as_f64();

    if num.is_nan() {
        Ok(Value::Number(f64::NAN))
    } else if num.is_infinite() {
        Ok(Value::Number(num))
    } else if num == 0.0 && num.is_sign_negative() {
        Ok(Value::Number(-0.0))
    } else {
        let floored = num.floor();
        if floored >= i32::MIN as f64 && floored <= i32::MAX as f64 {
            Ok(Value::Integer(floored as i32))
        } else {
            Ok(Value::Number(floored))
        }
    }
}
