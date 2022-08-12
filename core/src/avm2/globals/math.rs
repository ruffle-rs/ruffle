//! `Math` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;

macro_rules! math_wrap_std {
    ($std:expr) => {
        |activation, _this, args| -> Result<Value<'_>, Error> {
            if let Some(input) = args.get(0) {
                Ok($std(input.coerce_to_number(activation)?).into())
            } else {
                Ok(f64::NAN.into())
            }
        }
    };
}

/// Implements `Math`'s instance initializer.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // TODO: Replace with actual error type.
    Err("TypeError: Error #1076: Math is not a constructor.".into())
}

/// Implements `Math`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `Math`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "Math"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Math instance initializer>", mc),
        Method::from_builtin(class_init, "<Math class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    use std::f64::consts::*;
    const CONSTANTS: &[(&str, f64)] = &[
        ("E", E),
        ("LN10", LN_10),
        ("LN2", LN_2),
        ("LOG10E", LOG10_E),
        ("LOG2E", LOG2_E),
        ("PI", PI),
        ("SQRT1_2", FRAC_1_SQRT_2),
        ("SQRT2", SQRT_2),
    ];
    write.define_public_constant_number_class_traits(CONSTANTS);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("atan2", atan2),
        ("max", max),
        ("min", min),
        ("pow", pow),
        ("random", random),
        ("round", round),
        ("abs", math_wrap_std!(f64::abs)),
        ("acos", math_wrap_std!(f64::acos)),
        ("asin", math_wrap_std!(f64::asin)),
        ("atan", math_wrap_std!(f64::atan)),
        ("ceil", math_wrap_std!(f64::ceil)),
        ("cos", math_wrap_std!(f64::cos)),
        ("exp", math_wrap_std!(f64::exp)),
        ("floor", math_wrap_std!(f64::floor)),
        ("log", math_wrap_std!(f64::ln)),
        ("sin", math_wrap_std!(f64::sin)),
        ("sqrt", math_wrap_std!(f64::sqrt)),
        ("tan", math_wrap_std!(f64::tan)),
    ];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    class
}

fn round<'gc>(
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

fn atan2<'gc>(
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

fn max<'gc>(
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

fn min<'gc>(
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

fn pow<'gc>(
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
