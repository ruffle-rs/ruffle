use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};

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

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "SQRT2" => value(consts::SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SQRT1_2" => value(consts::FRAC_1_SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PI" => value(consts::PI; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG2E" => value(consts::LOG2_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG10E" => value(consts::LOG10_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN2" => value(consts::LN_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN10" => value(consts::LN_10; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "E" => value(consts::E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "abs" => method(wrap_std!(f64::abs); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "min" => method(min; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "max" => method(max; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sin" => method(wrap_std!(f64::sin); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "cos" => method(wrap_std!(f64::cos); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan2" => method(atan2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "tan" => method(wrap_std!(f64::tan); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "exp" => method(wrap_std!(f64::exp); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "log" => method(wrap_std!(f64::ln); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sqrt" => method(wrap_std!(f64::sqrt); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "round" => method(round; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "random" => method(random; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "floor" => method(wrap_std!(f64::floor); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ceil" => method(wrap_std!(f64::ceil); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan" => method(wrap_std!(f64::atan); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "asin" => method(wrap_std!(f64::asin); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "acos" => method(wrap_std!(f64::acos); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "pow" => method(pow; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let math = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(math, OBJECT_DECLS(context));
    math
}

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
    if let Some(y) = args.get(0)
        && let Some(x) = args.get(1)
    {
        let x = x.coerce_to_f64(activation)?;
        if x.is_nan() {
            return Ok(f64::NAN.into());
        }
        return Ok(y.coerce_to_f64(activation)?.powf(x).into());
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
    let rand = activation.context.rng.generate_random_number();
    Ok(((rand as f64) / (MAX_VAL as f64 + 1f64)).into())
}
