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

/// Strips leading whitespace characters.
fn skip_spaces(s: &mut &WStr) {
    *s = s.trim_start_matches(|c| {
        matches!(
            c,
            0x20 | 0x09 | 0x0d | 0x0a | 0x0c | 0x0b | 0x2000
                ..=0x200b | 0x2028 | 0x2029 | 0x205f | 0x3000
        )
    });
}

/// Consumes an optional sign character.
/// Returns whether a minus sign was consumed.
fn parse_sign(s: &mut &WStr) -> bool {
    if let Some(after_sign) = s.strip_prefix(b'-') {
        *s = after_sign;
        true
    } else if let Some(after_sign) = s.strip_prefix(b'+') {
        *s = after_sign;
        false
    } else {
        false
    }
}

/// Converts a `WStr` to an `f64`.
///
/// This function might fail for some invalid inputs, by returning `None`.
///
/// `strict` typically tells whether to behave like `Number()` or `parseFloat()`:
/// * `strict == true` fails on trailing garbage, but interprets blank strings (which are empty or consist only of whitespace characters) as zero.
/// * `strict == false` ignores trailing garbage, but fails on blank strings.
fn string_to_f64(mut s: &WStr, swf_version: u8, strict: bool) -> Option<f64> {
    // Allow leading whitespace characters.
    skip_spaces(&mut s);

    if s.is_empty() {
        // A blank string. Handle it as described above.
        return if strict { Some(0.0) } else { None };
    }

    let is_negative = parse_sign(&mut s);
    let after_sign = s;

    // Count digits before decimal point.
    s = s.trim_start_matches(|c| (c as u8).is_ascii_digit());
    let mut total_digits = after_sign.len() - s.len();

    // Count digits after decimal point.
    if let Some(after_dot) = s.strip_prefix(b'.') {
        s = after_dot;
        s = s.trim_start_matches(|c| (c as u8).is_ascii_digit());
        total_digits += after_dot.len() - s.len();
    }

    // Handle exponent.
    let mut exponent: i32 = 0;
    if let Some(after_e) = s.strip_prefix(b"eE".as_ref()) {
        s = after_e;
        let exponent_is_negative = parse_sign(&mut s);

        // Fail if string ends with "e-" with no exponent value specified.
        if exponent_is_negative && s.is_empty() {
            return None;
        }

        s = s.trim_start_matches(|c| match (c as u8 as char).to_digit(10) {
            Some(digit) => {
                exponent = exponent.wrapping_mul(10);
                exponent = exponent.wrapping_add(digit as i32);
                true
            }
            None => false,
        });

        // Apply exponent sign.
        if exponent_is_negative {
            exponent = exponent.wrapping_neg();
        }
    }

    // Allow trailing whitespace characters.
    skip_spaces(&mut s);

    // If we got no digits, check for Infinity/-Infinity, else fail.
    if total_digits == 0 {
        if let Some(after_infinity) = s.strip_prefix(WStr::from_units(b"Infinity")) {
            s = after_infinity;

            // Allow end of string or a whitespace, and fail otherwise.
            if !s.is_empty() {
                skip_spaces(&mut s);
                if s == after_infinity {
                    return None;
                }
            }

            let result = if is_negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            return Some(result);
        }
        return None;
    }

    // If we got digits, but we're in strict mode and not at end of string (or at null character), fail.
    if strict && !s.is_empty() && !s.starts_with(b'\0') {
        return None;
    }

    // Bug compatibility: https://bugzilla.mozilla.org/show_bug.cgi?id=513018
    let s = if swf_version >= 11 {
        &after_sign[..after_sign.len() - s.len()]
    } else {
        after_sign
    };

    let mut result = if total_digits > 15 {
        // With more than 15 digits, avmplus uses integer arithmetic to avoid rounding errors.
        let mut result: i64 = 0;
        let mut decimal_digits = -1;
        for c in s {
            if let Some(digit) = (c as u8 as char).to_digit(10) {
                if decimal_digits != -1 {
                    decimal_digits += 1;
                }

                result *= 10;
                result += i64::from(digit);
            } else if c == b'.' as u16 {
                decimal_digits = 0;
            } else {
                break;
            }
        }

        if decimal_digits > 0 {
            exponent -= decimal_digits;
        }

        if exponent > 0 {
            result *= i64::pow(10, exponent as u32);
        }

        result as f64
    } else {
        let mut result = 0.0;
        let mut decimal_digits = -1;
        for c in s {
            if let Some(digit) = (c as u8 as char).to_digit(10) {
                if decimal_digits != -1 {
                    decimal_digits += 1;
                }

                result *= 10.0;
                result += digit as f64;
            } else if c == b'.' as u16 {
                decimal_digits = 0;
            } else {
                break;
            }
        }

        if decimal_digits > 0 {
            exponent -= decimal_digits;
        }

        if exponent > 0 {
            result *= f64::powi(10.0, exponent);
        }

        result
    };

    if exponent < 0 {
        if exponent < -307 {
            let diff = exponent + 307;
            result /= f64::powi(10.0, -diff);
            exponent = -307;
        }
        result /= f64::powi(10.0, -exponent);
    }

    // Apply sign.
    if is_negative {
        result = -result;
    }

    Some(result)
}

pub fn parse_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(value) = args.get(0) {
        let string = value.coerce_to_string(activation)?;
        let swf_version = activation.context.swf.version();
        if let Some(result) = string_to_f64(&string, swf_version, false) {
            return Ok(result.into());
        }
    }

    Ok(f64::NAN.into())
}
