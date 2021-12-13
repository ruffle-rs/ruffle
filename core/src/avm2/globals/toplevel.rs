//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::WStr;

pub fn trace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    match args {
        [] => activation.context.avm_trace(""),
        [arg] => {
            let msg = arg.coerce_to_string(activation)?;
            activation.context.avm_trace(&msg.to_utf8_lossy());
        }
        args => {
            let strings = args
                .iter()
                .map(|a| a.coerce_to_string(activation))
                .collect::<Result<Vec<_>, _>>()?;
            let msg = crate::string::join(&strings, &WStr::from_units(b" "));
            activation.context.avm_trace(&msg.to_utf8_lossy());
        }
    }

    Ok(Value::Undefined)
}

pub fn is_finite<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_number(activation)?.is_finite().into())
    } else {
        Ok(false.into())
    }
}

pub fn is_nan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_number(activation)?.is_nan().into())
    } else {
        Ok(true.into())
    }
}

pub fn parse_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let string = match args.get(0) {
        None => return Ok(f64::NAN.into()),
        Some(Value::Undefined) => Value::Null,
        Some(v) => *v,
    }
    .coerce_to_string(activation)?;

    let radix = if let Some(val) = args.get(1) {
        Some(val.coerce_to_u32(activation)?)
    } else {
        None
    };
    let radix = match radix {
        Some(r @ 2..=36) => Some(r as u32),
        Some(0) => None,
        Some(_) => return Ok(f64::NAN.into()),
        None => None,
    };

    let string = string.as_wstr();

    // Strip spaces.
    let string = string.trim_start_matches(b"\t\n\r ".as_ref());

    if string.is_empty() {
        return Ok(f64::NAN.into());
    }

    let (sign, string) = match u8::try_from(string.get(0).unwrap()) {
        Ok(b'+') => (1.0, &string[1..]),
        Ok(b'-') => (-1.0, &string[1..]),
        _ => (1.0, string),
    };

    fn starts_with_0x(string: &WStr) -> bool {
        if string.get(0) == Some(b'0' as u16) {
            let x_char = string.get(1);
            x_char == Some(b'x' as u16) || x_char == Some(b'X' as u16)
        } else {
            false
        }
    }

    let (radix, string) = match radix {
        None => {
            if starts_with_0x(string) {
                (16, &string[2..])
            } else {
                (10, string)
            }
        }
        Some(16) => {
            if starts_with_0x(string) {
                (16, &string[2..])
            } else {
                (16, string)
            }
        }
        Some(radix) => (radix, string),
    };

    let mut empty = true;
    let mut result = 0.0f64;
    for chr in string {
        let digit = u8::try_from(chr)
            .ok()
            .and_then(|c| (c as char).to_digit(radix));
        if let Some(digit) = digit {
            result = result * radix as f64 + digit as f64;
            empty = false;
        } else {
            break;
        }
    }

    if empty {
        Ok(f64::NAN.into())
    } else {
        Ok(result.copysign(sign).into())
    }
}

pub fn parse_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let s = if let Some(val) = args.get(0) {
        val.coerce_to_string(activation)?
    } else {
        return Ok(f64::NAN.into());
    };

    let s = s.trim_start();

    if s.starts_with(WStr::from_units(b"Infinity")) || s.starts_with(WStr::from_units(b"+Infinity"))
    {
        return Ok(f64::INFINITY.into());
    } else if s.starts_with(WStr::from_units(b"-Infinity")) {
        return Ok((-f64::INFINITY).into());
    }

    // TODO: this reuses logic from AVM1,
    // which is generally much more lenient.
    // There are some cases we should accept (like "- Infinity", but not "- 1")
    // And some we should not (like "InfinityXYZ")
    use crate::avm1::globals::parse_float_impl;
    Ok(parse_float_impl(s, false).into())
}
