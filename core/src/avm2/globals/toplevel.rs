//! Global scope built-ins

use ruffle_wstr::Units;

use crate::avm2::activation::Activation;
use crate::avm2::error::{uri_error, Error};
use crate::avm2::object::Object;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::string::{AvmString, WStr, WString};
use ruffle_wstr::Integer;
use std::fmt::Write;

pub fn trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let val = args.get_f64(activation, 0)?;

    Ok(val.is_finite().into())
}

pub fn is_na_n<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let val = args.get_f64(activation, 0)?;

    Ok(val.is_nan().into())
}

pub fn parse_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let string = args.get_string(activation, 0)?;
    let radix = args.get_i32(activation, 1)?;

    let result = crate::avm2::value::string_to_int(&string, radix, false);
    Ok(result.into())
}

pub fn parse_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let string = args.get_string(activation, 0)?;
    let swf_version = activation.context.swf.version();

    if let Some(result) = crate::avm2::value::string_to_f64(&string, swf_version, false) {
        Ok(result.into())
    } else {
        Ok(f64::NAN.into())
    }
}

pub fn is_xml_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_value(0);
    if matches!(name, Value::Undefined | Value::Null) {
        return Ok(false.into());
    }

    let name = name.coerce_to_string(activation)?;

    Ok(crate::avm2::e4x::is_xml_name(name).into())
}

pub fn escape<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_string(activation, 0)?;

    let mut output = WString::new();

    // Characters that are not escaped, sourced from as3 docs
    let not_converted =
        WStr::from_units(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@-_.*+/");

    for x in value.iter() {
        if not_converted.contains(x) {
            output.push(x);
        } else {
            let encode = if x <= u8::MAX.into() {
                format!("%{x:02X}")
            } else {
                format!("%u{x:04X}")
            };
            output.push_str(WStr::from_units(encode.as_bytes()));
        }
    }

    Ok(AvmString::new(activation.context.gc_context, output).into())
}

pub fn unescape<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_string(activation, 0)?;

    let mut output = WString::new();
    let mut index = 0;
    while let Some(byte) = value.get(index) {
        index += 1;
        if byte != b'%' as u16 {
            output.push(byte);
            continue;
        }

        let prev_index = index;
        let len = match value.get(index) {
            // 0x75 == 'u'
            Some(0x75) => {
                // increment one to consume the 'u'
                index += 1;
                4
            }
            _ => 2,
        };

        if let Some(x) = value
            .slice(index..)
            .and_then(|v| v.slice(..len))
            .and_then(|v| u32::from_wstr_radix(v, 16).ok())
        {
            // NOTE: Yes, unpaired surrogates are allowed
            output.push(x as u16);
            index += len;
        } else {
            output.push(b'%' as u16);
            index = prev_index;
        }
    }
    Ok(AvmString::new(activation.context.gc_context, output).into())
}

pub fn encode_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    encode_utf8_with_exclusions(
        activation,
        args,
        // Characters that are not escaped, sourced from as3 docs
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@;/?:@&=+$,#-_.!~*'()",
    )
}

pub fn encode_uri_component<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    encode_utf8_with_exclusions(
        activation,
        args,
        // Characters that are not escaped, sourced from as3 docs
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_.!~*'()",
    )
}

fn encode_utf8_with_exclusions<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
    not_converted: &str,
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args.get_string(activation, 0)?;
    let mut output = String::new();

    let input_string = match input.units() {
        // Latin-1 values map directly to unicode codepoints,
        // so we can directly convert to a `char`
        Units::Bytes(bytes) => bytes.iter().map(|b| *b as char).collect(),
        Units::Wide(wide) => String::from_utf16_lossy(wide),
    };

    for x in input_string.chars() {
        if not_converted.contains(x) {
            output.push(x);
        } else {
            let mut bytes = [0; 4];
            let utf8_bytes = x.encode_utf8(&mut bytes);
            let mut encoded = String::new();
            // Each byte in the utf-8 encoding is encoded as a hex value
            for byte in utf8_bytes.bytes() {
                write!(encoded, "%{x:02X}", x = byte).unwrap();
            }
            output.push_str(&encoded);
        }
    }

    Ok(AvmString::new_utf8(activation.context.gc_context, output).into())
}

pub fn decode_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    decode(
        activation,
        args,
        // Characters that are reserved, sourced from as3 docs
        "#$&+,/:;=?@",
        "decodeURI",
    )
}

pub fn decode_uri_component<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    decode(activation, args, "", "decodeURIComponent")
}

fn handle_percent<I>(chars: &mut I) -> Option<u8>
where
    I: Iterator<Item = Result<char, std::char::DecodeUtf16Error>>,
{
    let high = chars.next()?.ok()?.to_digit(16)?;
    let low = chars.next()?.ok()?.to_digit(16)?;
    Some(low as u8 | ((high as u8) << 4))
}

// code derived from flash.utils.unescapeMultiByte
// FIXME: support bugzilla #538107
fn decode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
    reserved_set: &str,
    func_name: &str,
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_string(activation, 0)?;

    let mut output = WString::new();
    let mut chars = value.chars();
    let mut bytes = Vec::with_capacity(4);

    while let Some(c) = chars.next() {
        let Ok(c) = c else {
            return Err(Error::AvmError(uri_error(
                activation,
                &format!("Error #1052: Invalid URI passed to {func_name} function."),
                1052,
            )?));
        };

        if c != '%' {
            output.push_char(c);
            continue;
        }

        bytes.clear();
        let Some(byte) = handle_percent(&mut chars) else {
            return Err(Error::AvmError(uri_error(
                activation,
                &format!("Error #1052: Invalid URI passed to {func_name} function."),
                1052,
            )?));
        };
        bytes.push(byte);
        if (byte & 0x80) != 0 {
            let n = byte.leading_ones();

            if n == 1 || n > 4 {
                return Err(Error::AvmError(uri_error(
                    activation,
                    &format!("Error #1052: Invalid URI passed to {func_name} function."),
                    1052,
                )?));
            }

            for _ in 1..n {
                if chars.next() != Some(Ok('%')) {
                    return Err(Error::AvmError(uri_error(
                        activation,
                        &format!("Error #1052: Invalid URI passed to {func_name} function."),
                        1052,
                    )?));
                }; // consume %

                let Some(byte) = handle_percent(&mut chars) else {
                    return Err(Error::AvmError(uri_error(
                        activation,
                        &format!("Error #1052: Invalid URI passed to {func_name} function."),
                        1052,
                    )?));
                };

                if (byte & 0xC0) != 0x80 {
                    return Err(Error::AvmError(uri_error(
                        activation,
                        &format!("Error #1052: Invalid URI passed to {func_name} function."),
                        1052,
                    )?));
                }

                bytes.push(byte);
            }
        }

        let Ok(decoded) = std::str::from_utf8(&bytes) else {
            return Err(Error::AvmError(uri_error(
                activation,
                &format!("Error #1052: Invalid URI passed to {func_name} function."),
                1052,
            )?));
        };
        if reserved_set.contains(decoded) {
            for byte in &bytes {
                write!(output, "%{x:02X}", x = byte).unwrap();
            }
        } else {
            output.push_utf8(decoded);
        }
    }

    Ok(AvmString::new(activation.context.gc_context, output).into())
}
