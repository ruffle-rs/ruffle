//! `Math` impl

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use rand::Rng;

macro_rules! wrap_std {
    ($name:ident, $std:expr) => {
        pub fn $name<'gc>(
            activation: &mut Activation<'_, 'gc, '_>,
            _this: Option<Object<'gc>>,
            args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error> {
            if let Some(input) = args.get(0) {
                Ok($std(input.coerce_to_number(activation)?).into())
            } else {
                Ok(f64::NAN.into())
            }
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

pub fn round<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(x) = args.get(0) {
        let x = x.coerce_to_number(activation)?;
        // Note that Flash Math.round always rounds toward infinity,
        // unlike Rust f32::round which rounds away from zero.
        let ret = (x + 0.5).floor();
        return Ok(ret.into());
    }
    Ok(f64::NAN.into())
}

pub fn atan2<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let y = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    let x = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    Ok(f64::atan2(y, x).into())
}

pub fn max<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let n = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    let p = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    Ok(f64::powf(n, p).into())
}

pub fn random<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(activation.context.rng.gen_range(0.0f64..1.0f64).into())
}
