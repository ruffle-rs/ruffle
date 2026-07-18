//! `Number` impl

use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_1002, make_error_1003};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error};
use ruffle_macros::istr;

pub fn number_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number_value = args
        .get_optional(0)
        .unwrap_or(Value::Integer(0))
        .coerce_to_number(activation)?;

    Ok(number_value.into())
}

macro_rules! define_math_functions {
    ($($name:ident),* $(,)?) => {
        $(
            pub fn $name<'gc>(
                activation: &mut Activation<'_, 'gc>,
                this: Value<'gc>,
                args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                crate::avm2::globals::math::$name(activation, this, args)
            }
        )*
    };
}

define_math_functions!(
    abs, acos, asin, atan, atan2, ceil, cos, exp, floor, log, max, min, pow, random, round, sin,
    sqrt, tan
);

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get_optional(0)
        .unwrap_or(Value::Number(0.0))
        .coerce_to_number(activation)?
        .into())
}

fn to_exponential(number: f64, digits: usize) -> String {
    match (number, digits) {
        (0.0, 0) => "1e-15".to_owned(),
        (0.0, _) => format!("0.{}e-16", "0".repeat(digits)),
        _ => format!("{number:.digits$e}")
            .replace('e', "e+")
            .replace("e+-", "e-")
            .replace("e+0", ""),
    }
}

fn to_fixed(number: f64, digits: usize) -> String {
    format!("{0:.1$}", number + 0.0, digits)
}

fn to_precision(number: f64, wanted_digits: usize) -> String {
    let mut available_digits = number.abs().log10().floor();
    if available_digits.is_nan() || available_digits.is_infinite() {
        available_digits = 1.0;
    }

    let precision = (number * 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0)).floor()
        / 10.0_f64.powf(wanted_digits as f64 - available_digits - 1.0);

    if (wanted_digits as f64) <= available_digits {
        format!(
            "{}e{}{}",
            precision / 10.0_f64.powf(available_digits),
            if available_digits < 0.0 { "-" } else { "+" },
            available_digits.abs()
        )
    } else {
        format!("{precision}")
    }
}

/// Implements `Number._convert`
pub fn _convert<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = args.get_f64(0);
    let digits = args.get_i32(1);

    // mode 0: toExponential
    // mode 1: toFixed
    // mode 2: toPrecision
    let mode = args.get_i32(2);

    if !(0..=20).contains(&if mode == 2 { digits - 1 } else { digits }) {
        return Err(make_error_1002(activation));
    }

    if !number.is_finite() {
        return Ok(Value::from(number).coerce_to_string(activation)?.into());
    }

    let digits = digits as usize;
    let result = match mode {
        0 => to_exponential(number, digits),
        1 => to_fixed(number, digits),
        2 => to_precision(number, digits),
        _ => unreachable!(),
    };

    Ok(AvmString::new_utf8(activation.gc(), result).into())
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
        return Ok(istr!("NaN"));
    }

    if number.is_infinite() {
        if number < 0.0 {
            return Ok(istr!("-Infinity"));
        } else if number > 0.0 {
            return Ok(istr!("Infinity"));
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

    Ok(AvmString::new_utf8(activation.gc(), formatted))
}

/// Implements `Number.prototype.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = this.as_f64();

    let radix = args.get_value(0).coerce_to_i32(activation)?;

    if radix < 2 || radix > 36 {
        return Err(make_error_1003(activation, radix));
    }

    Ok(print_with_radix(activation, number, radix as usize)?.into())
}

/// Implements `Number.prototype.valueOf`
pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this)
}
