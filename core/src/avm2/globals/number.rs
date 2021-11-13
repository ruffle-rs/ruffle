//! `Number` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{primitive_allocator, Object, TObject};
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
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
            let number = this.coerce_to_number(activation)?;
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
                    .replace("e+-", "e-"),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Number.toFixed`
pub fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            let number = this.coerce_to_number(activation)?;
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

    Ok(Value::Undefined)
}

/// Implements `Number.toPrecision`
pub fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            let number = this.coerce_to_number(activation)?;
            let wanted_digits = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;

            if wanted_digits < 1 || wanted_digits > 21 {
                return Err("toPrecision can only print with 1 through 21 digits.".into());
            }

            let available_digits = number.log10().floor();
            let precision = (number * 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0))
                .floor()
                / 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0);

            if (wanted_digits as f64) <= available_digits {
                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!(
                        "{}e{}{}",
                        precision / 10.0_f64.powf(available_digits),
                        if available_digits < 0.0 { "-" } else { "+" },
                        available_digits.abs()
                    ),
                )
                .into());
            } else {
                return Ok(
                    AvmString::new_utf8(activation.context.gc_context, format!("{}", precision)).into(),
                );
            }
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
        ("MIN_VALUE", f64::MIN),
        ("NaN", f64::NAN),
        ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
        ("POSITIVE_INFINITY", f64::INFINITY),
    ];
    write.define_public_constant_number_class_traits(CLASS_CONSTANTS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toLocaleString", to_locale_string),
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
