//! `Number` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error};
use gc_arena::{GcCell, MutationContext};

/// Implements `Number`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut prim) = this.as_primitive_mut(activation.context.gc_context) {
            if matches!(*prim, Value::Undefined | Value::Null) {
                *prim = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Number(0.0))
                    .coerce_to_number(activation)?
                    .into();
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Number`'s native instance initializer.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Number`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut number_proto = this
            .get_property(this, &QName::dynamic_name("prototype").into(), activation)?
            .coerce_to_object(activation)?;
        let scope = activation.create_scopechain();
        let gc_context = activation.context.gc_context;
        let this_class = this.as_class_object().unwrap();

        number_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toExponential"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_exponential, "toExponential", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
        number_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toFixed"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_fixed, "toFixed", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
        number_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toPrecision"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_precision, "toPrecision", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
        number_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toString"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_string, "toString", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
        number_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "valueOf"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(value_of, "valueOf", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `Number.toLocaleString`
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            return Ok(this.coerce_to_string(activation)?.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Number.toExponential`
pub fn to_exponential<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Number(number) = this.clone() {
                let digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Unsigned(0))
                    .coerce_to_u32(activation)? as usize;

                if digits > 20 {
                    return Err("toExponential can only print with 0 through 20 digits.".into());
                }

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("{0:.1$e}", number, digits)
                        .replace("e", "e+")
                        .replace("e+-", "e-")
                        .replace("e+0", ""),
                )
                .into());
            }
        }
    }

    Err("Number.prototype.toExponential has been called on an incompatible object".into())
}

/// Implements `Number.toFixed`
pub fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Number(number) = this.clone() {
                let digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Unsigned(0))
                    .coerce_to_u32(activation)? as usize;

                if digits > 20 {
                    return Err("toFixed can only print with 0 through 20 digits.".into());
                }

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("{0:.1$}", number, digits),
                )
                .into());
            }
        }
    }

    Err("Number.prototype.toFixed has been called on an incompatible object".into())
}

pub fn print_with_precision<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    number: f64,
    wanted_digits: usize,
) -> Result<AvmString<'gc>, Error> {
    let mut available_digits = number.abs().log10().floor();
    if available_digits.is_nan() || available_digits.is_infinite() {
        available_digits = 1.0;
    }

    let precision = (number * 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0)).floor()
        / 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0);

    if (wanted_digits as f64) <= available_digits {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            format!(
                "{}e{}{}",
                precision / 10.0_f64.powf(available_digits),
                if available_digits < 0.0 { "-" } else { "+" },
                available_digits.abs()
            ),
        ))
    } else {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            format!("{}", precision),
        ))
    }
}

/// Implements `Number.toPrecision`
pub fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Number(number) = this.clone() {
                let wanted_digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Unsigned(0))
                    .coerce_to_u32(activation)? as usize;

                if wanted_digits < 1 || wanted_digits > 21 {
                    return Err("toPrecision can only print with 1 through 21 digits.".into());
                }

                return Ok(print_with_precision(activation, number, wanted_digits)?.into());
            }
        }
    }

    Err("Number.prototype.toPrecision has been called on an incompatible object".into())
}

pub fn print_with_radix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    mut number: f64,
    radix: usize,
) -> Result<AvmString<'gc>, Error> {
    if radix == 10 {
        return Value::from(number).coerce_to_string(activation);
    }

    let mut digits = vec![];
    let sign = number.signum();
    number = number.abs();

    loop {
        let digit = number % radix as f64;
        number /= radix as f64;

        const DIGIT_CHARS: [char; 36] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];

        digits.push(*DIGIT_CHARS.get(digit as usize).unwrap());

        if number < 1.0 {
            break;
        }
    }

    if sign < 0.0 {
        digits.push('-');
    }

    let formatted: String = digits.into_iter().rev().collect();

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        formatted,
    ))
}

/// Implements `Number.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Number(number) = this.clone() {
                let radix = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Unsigned(10))
                    .coerce_to_u32(activation)? as usize;

                if radix < 2 || radix > 36 {
                    return Err("toString can only print in bases 2 thru 36.".into());
                }

                return Ok(print_with_radix(activation, number, radix)?.into());
            }
        }
    }

    Err("Number.prototype.toString has been called on an incompatible object".into())
}

/// Implements `Number.valueOf`
pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            return Ok(this.clone());
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Number`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "Number"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Number instance initializer>", mc),
        Method::from_builtin(class_init, "<Number class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Number native instance initializer>",
        mc,
    ));

    const CLASS_CONSTANTS: &[(&str, f64)] = &[
        ("MAX_VALUE", f64::MAX),
        ("MIN_VALUE", f64::MIN_POSITIVE),
        ("NaN", f64::NAN),
        ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
        ("POSITIVE_INFINITY", f64::INFINITY),
    ];
    write.define_public_constant_number_class_traits(CLASS_CONSTANTS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("toLocaleString", to_locale_string)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    class
}
