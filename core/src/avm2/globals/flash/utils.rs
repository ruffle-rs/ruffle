//! `flash.utils` namespace

use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};
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
    _this: Object<'gc>,
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
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setInterval: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setInterval: not enough arguments")
            .as_object()
            .ok_or("setInterval: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setInterval: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        false,
    )))
}

/// Implements `flash.utils.clearInterval`
pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearInterval: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.setTimeout`
pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setTimeout: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setTimeout: not enough arguments")
            .as_object()
            .ok_or("setTimeout: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setTimeout: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        true,
    )))
}

/// Implements `flash.utils.clearTimeout`
pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearTimeout: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.escapeMultiByte`
pub fn escape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
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
    Ok(AvmString::new(activation.context.gc_context, result).into())
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
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
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
    let v = AvmString::new(activation.context.gc_context, buf);
    Ok(v.into())
}

/// Implements `flash.utils.getQualifiedClassName`
pub fn get_qualified_class_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This is a native method, which enforces the argument count.
    let val = args[0];
    match val {
        Value::Null => return Ok("null".into()),
        Value::Undefined => return Ok("void".into()),
        _ => {}
    }
    let obj = val.coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class.inner_class_definition(),
        None => obj.instance_class(),
    };

    Ok(class
        .name()
        .to_qualified_name(activation.context.gc_context)
        .into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_superclass_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class.inner_class_definition(),
        None => obj.instance_class(),
    };

    if let Some(super_class) = class.super_class() {
        Ok(super_class
            .name()
            .to_qualified_name(activation.context.gc_context)
            .into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements native method `flash.utils.getDefinitionByName`
pub fn get_definition_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation
        .caller_domain()
        .expect("Missing caller domain in getDefinitionByName");
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    appdomain.get_defined_value_handling_vector(activation, name)
}
