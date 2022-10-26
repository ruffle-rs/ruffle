//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WStr, WString};

pub fn trace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
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
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
) -> Result<Value<'gc>, Error<'gc>> {
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
) -> Result<Value<'gc>, Error<'gc>> {
    let string = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined => "null".into(),
        value => value.coerce_to_string(activation)?,
    };

    let radix = match args.get(1) {
        Some(value) => value.coerce_to_i32(activation)?,
        None => 0,
    };

    let result = crate::avm2::value::string_to_int(&string, radix, false);
    Ok(result.into())
}

pub fn parse_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(value) = args.get(0) {
        let string = value.coerce_to_string(activation)?;
        let swf_version = activation.context.swf.version();
        if let Some(result) = crate::avm2::value::string_to_f64(&string, swf_version, false) {
            return Ok(result.into());
        }
    }

    Ok(f64::NAN.into())
}

pub fn escape<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = match args.first() {
        None => return Ok("undefined".into()),
        Some(Value::Undefined) => return Ok("null".into()),
        Some(value) => value,
    };

    let mut output = WString::new();

    // Characters that are not escaped, sourced from as3 docs
    let not_converted =
        WStr::from_units(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@-_.*+/");

    for x in value.coerce_to_string(activation)?.iter() {
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
