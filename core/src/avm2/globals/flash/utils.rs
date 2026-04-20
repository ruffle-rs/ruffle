//! `flash.utils` namespace

use crate::avm2::error::make_error_1507;
use crate::avm2::globals::avmplus::instance_class_describe_type;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;
use crate::string::WString;
use std::fmt::Write;
use web_time::Instant;

pub mod byte_array;
pub mod dictionary;
pub mod proxy;
pub mod timer;

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((Instant::now()
        .duration_since(activation.context.start_time)
        .as_millis() as u32)
        .into())
}

/// Implements `flash.utils.setInterval`
pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let closure = args.try_get_function(0);
    let interval = args.get_f64(1);
    let params = &args[2..];

    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure,
        params: params.to_vec(),
    };

    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        false,
    )))
}

/// Implements `flash.utils.clearInterval`
pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args.get_u32(0);
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.setTimeout`
pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let closure = args.try_get_function(0);
    let interval = args.get_f64(1);
    let params = &args[2..];

    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure,
        params: params.to_vec(),
    };

    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        true,
    )))
}

/// Implements `flash.utils.clearTimeout`
pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args.get_u32(0);
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.escapeMultiByte`
pub fn escape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args.get_string(activation, 0);

    let utf8 = s.as_wstr().to_utf8_lossy();
    let mut result = WString::new();
    for byte in utf8.as_bytes() {
        if *byte == 0 {
            break;
        }
        if byte.is_ascii_alphanumeric() {
            result.push_byte(*byte);
        } else {
            let _ = write!(&mut result, "%{byte:02X}");
        }
    }
    Ok(AvmString::new(activation.gc(), result).into())
}

fn handle_percent<I>(chars: &mut I) -> Option<u8>
where
    I: Iterator<Item = char>,
{
    let high = chars.next()?.to_digit(16)? as u8;
    let low = chars.next()?.to_digit(16)? as u8;
    Some(low | (high << 4))
}

/// Implements `flash.utils.unescapeMultiByte`
pub fn unescape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args.get_string(activation, 0);

    let bs = s.as_wstr();
    let mut buf = WString::new();
    let chars = bs.chars().map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER));

    let mut chars = chars.peekable();
    let mut utf8_bytes = Vec::new();
    while let Some(c) = chars.next() {
        if c == '\0' {
            break;
        }
        if c == '%' {
            while let Some(b) = handle_percent(&mut chars) {
                utf8_bytes.push(b);
                if !matches!(chars.peek(), Some('%')) {
                    break;
                }
                chars.next();
            }
            buf.push_utf8_bytes(&utf8_bytes);
            utf8_bytes.clear();
            continue;
        }

        buf.push_char(c);
    }
    let v = AvmString::new(activation.gc(), buf);
    Ok(v.into())
}

/// Implements `flash.utils.getQualifiedClassName`
pub fn get_qualified_class_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_value(0);
    let class = instance_class_describe_type(activation, value);

    let mc = activation.gc();
    Ok(class.dollar_removed_name(mc).to_qualified_name(mc).into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_superclass_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args.get_value(0);

    let class = match value.as_object().and_then(|o| o.as_class_object()) {
        Some(class) => class.inner_class_definition(),
        None => instance_class_describe_type(activation, value),
    };

    if let Some(super_class) = class.super_class() {
        Ok(super_class.name().to_qualified_name(activation.gc()).into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements native method `flash.utils.getDefinitionByName`
pub fn get_definition_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation
        .caller_domain()
        .expect("Missing caller domain in getDefinitionByName");
    let name = args.try_get_string(0);

    if let Some(name) = name {
        appdomain.get_defined_value_handling_vector(activation, name)
    } else {
        // For some reason this throws error #1507, so we can't use
        // `get_string_non_null` to get the argument
        Err(make_error_1507(activation, "name"))
    }
}
