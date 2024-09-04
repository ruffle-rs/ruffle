//! `Number` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{make_error_1002, make_error_1003, make_error_1004};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::QName;
use crate::avm2::{AvmString, Error};

/// Implements `Number`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Ok(Value::Undefined)
}

/// Implements `Number`'s native instance initializer.
fn super_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `Number`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let number_proto = this_class.prototype();

    number_proto.set_string_property_local(
        "toExponential",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_exponential, "toExponential", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_string_property_local(
        "toFixed",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_fixed, "toFixed", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_string_property_local(
        "toPrecision",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_precision, "toPrecision", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_string_property_local(
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_string_property_local(
        "toString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_string_property_local(
        "valueOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    number_proto.set_local_property_is_enumerable(gc_context, "toExponential".into(), false);
    number_proto.set_local_property_is_enumerable(gc_context, "toFixed".into(), false);
    number_proto.set_local_property_is_enumerable(gc_context, "toPrecision".into(), false);
    number_proto.set_local_property_is_enumerable(gc_context, "toLocaleString".into(), false);
    number_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    number_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get(0)
        .cloned()
        .unwrap_or(Value::Number(0.0))
        .coerce_to_number(activation)?
        .into())
}

/// Implements `Number.toExponential`
pub fn to_exponential<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let digits = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_i32(activation)?;

    if digits < 0 || digits > 20 {
        return Err(make_error_1002(activation));
    }

    let digits = digits as usize;

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!("{number:.digits$e}")
            .replace('e', "e+")
            .replace("e+-", "e-")
            .replace("e+0", ""),
    )
    .into())
}

/// Implements `Number.toFixed`
pub fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let digits = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_i32(activation)?;

    if digits < 0 || digits > 20 {
        return Err(make_error_1002(activation));
    }

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!("{0:.1$}", number, digits as usize),
    )
    .into())
}

pub fn print_with_precision<'gc>(
    activation: &mut Activation<'_, 'gc>,
    number: f64,
    wanted_digits: u32,
) -> Result<AvmString<'gc>, Error<'gc>> {
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
            format!("{precision}"),
        ))
    }
}

/// Implements `Number.toPrecision`
pub fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let wanted_digits = args.get(0).cloned().unwrap_or(Value::Integer(0));

    if matches!(wanted_digits, Value::Undefined) {
        return this.call_public_property("toString", &[], activation);
    }

    let wanted_digits = wanted_digits.coerce_to_i32(activation)?;

    if wanted_digits < 1 || wanted_digits > 21 {
        return Err(make_error_1002(activation));
    }

    Ok(print_with_precision(activation, number, wanted_digits as u32)?.into())
}

pub fn print_with_radix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mut number: f64,
    radix: usize,
) -> Result<AvmString<'gc>, Error<'gc>> {
    if radix == 10 {
        return Value::from(number).coerce_to_string(activation);
    }

    if number.is_nan() {
        return Ok("NaN".into());
    }

    if number.is_infinite() {
        if number < 0.0 {
            return Ok("-Infinity".into());
        } else if number > 0.0 {
            return Ok("Infinity".into());
        }
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

/// Implements `Number.prototype.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number_proto = activation.avm2().classes().number.prototype();
    if Object::ptr_eq(number_proto, this) {
        return Ok("0".into());
    }

    let number = if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(o) => o as f64,
            Value::Number(o) => o,
            _ => return Err(make_error_1004(activation, "Number.prototype.toString")),
        }
    } else {
        return Err(make_error_1004(activation, "Number.prototype.toString"));
    };

    let radix = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(10))
        .coerce_to_i32(activation)?;

    if radix < 2 || radix > 36 {
        return Err(make_error_1003(activation, radix));
    }

    Ok(print_with_radix(activation, number, radix as usize)?.into())
}

/// Implements `Number.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number_proto = activation.avm2().classes().number.prototype();
    if Object::ptr_eq(number_proto, this) {
        return Ok(0.into());
    }

    if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(_) => Ok(*this),
            Value::Number(_) => Ok(*this),
            _ => Err(make_error_1004(activation, "Number.prototype.valueOf")),
        }
    } else {
        Err(make_error_1004(activation, "Number.prototype.valueOf"))
    }
}

/// Construct `Number`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let class = Class::new(
        QName::new(namespaces.public_all(), "Number"),
        Some(activation.avm2().class_defs().object),
        Method::from_builtin(instance_init, "<Number instance initializer>", mc),
        Method::from_builtin(class_init, "<Number class initializer>", mc),
        activation.avm2().class_defs().class,
        mc,
    );

    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);
    class.set_instance_allocator(mc, primitive_allocator);
    class.set_super_init(
        mc,
        Method::from_builtin(super_init, "<Number native instance initializer>", mc),
    );
    class.set_call_handler(
        mc,
        Method::from_builtin(call_handler, "<Number call handler>", mc),
    );

    const CLASS_CONSTANTS_NUMBER: &[(&str, f64)] = &[
        ("MAX_VALUE", f64::MAX),
        ("MIN_VALUE", f64::MIN_POSITIVE),
        ("NaN", f64::NAN),
        ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
        ("POSITIVE_INFINITY", f64::INFINITY),
        ("E", std::f64::consts::E),
        ("PI", std::f64::consts::PI),
        ("SQRT2", std::f64::consts::SQRT_2),
        ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
        ("LN2", std::f64::consts::LN_2),
        ("LN10", std::f64::consts::LN_10),
        ("LOG2E", std::f64::consts::LOG2_E),
        ("LOG10E", std::f64::consts::LOG10_E),
    ];
    class.define_constant_number_class_traits(
        namespaces.public_all(),
        CLASS_CONSTANTS_NUMBER,
        activation,
    );

    const CLASS_CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    class.define_constant_int_class_traits(
        namespaces.public_all(),
        CLASS_CONSTANTS_INT,
        activation,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    class.define_builtin_instance_methods(mc, namespaces.as3, AS3_INSTANCE_METHODS);

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
