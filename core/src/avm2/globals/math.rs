//! `Math` impl

use crate::avm2::activation::Activation;
use crate::avm2::error::type_error;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::{ClassObject, Error};
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
    Ok(ret.into())
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
        } else if val > cur_max {
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
        } else if val < cur_min {
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
    let rand = activation.context.rng.random_range(0..MAX_VAL);
    Ok(((rand as f64) / (MAX_VAL as f64 + 1f64)).into())
}
