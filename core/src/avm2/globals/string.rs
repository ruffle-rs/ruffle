//! `String` impl

use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::regexp::{RegExp, RegExpFlags};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::{ArrayObject, ArrayStorage};
use crate::string::{AvmString, WString};

pub fn string_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let string_value = match args.get(0) {
        Some(arg) => arg.coerce_to_string(activation)?,
        None => istr!(""),
    };

    Ok(string_value.into())
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(arg) => arg.coerce_to_string(activation).map(Into::into),
        None => Ok(istr!("").into()),
    }
}

/// Implements `length` property's getter
pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::String(s) = this {
        return Ok(s.len().into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.charAt`
pub fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::String(s) = this {
        // This function takes Number, so if we use get_i32 instead of get_f64, the value may overflow.
        let n = args.get_f64(activation, 0)?;

        if n < 0.0 {
            return Ok(istr!("").into());
        }

        let index = if !n.is_nan() { n as usize } else { 0 };
        let ret = if let Some(c) = s.get(index) {
            activation.strings().make_char(c)
        } else {
            istr!("")
        };
        return Ok(ret.into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.charCodeAt`
pub fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::String(s) = this {
        // This function takes Number, so if we use coerce_to_i32 instead of coerce_to_number, the value may overflow.
        let n = args.get_f64(activation, 0)?;

        if n < 0.0 {
            return Ok(f64::NAN.into());
        }

        let index = if !n.is_nan() { n as usize } else { 0 };
        let ret = s.get(index).map(f64::from).unwrap_or(f64::NAN);
        return Ok(ret.into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.concat`
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut ret = this.coerce_to_string(activation)?;

    for arg in args {
        let s = arg.coerce_to_string(activation)?;
        ret = AvmString::concat(activation.gc(), ret, s);
    }

    Ok(ret.into())
}

/// Implements `String.fromCharCode`
pub fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut out = WString::with_capacity(args.len(), false);
    for arg in args {
        let i = arg.coerce_to_u32(activation)? as u16;
        out.push(i);
    }
    Ok(AvmString::new(activation.gc(), out).into())
}

/// Implements `String.indexOf`
pub fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let pattern = args.get_string(activation, 0)?;

    let start_index = args.get_i32(activation, 1)?.max(0) as usize;

    this.slice(start_index..)
        .and_then(|s| s.find(&pattern))
        .map(|i| Ok((i + start_index).into()))
        .unwrap_or_else(|| Ok((-1).into())) // Out of range or not found
}

/// Implements `String.lastIndexOf`
pub fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let pattern = args.get_string(activation, 0)?;

    let start_index = args.get_i32(activation, 1)?;

    let start_index = match usize::try_from(start_index) {
        Ok(n) => n + pattern.len(),
        Err(_) => return Ok((-1).into()), // Bail out on negative indices.
    };

    this.slice(..start_index)
        .unwrap_or(&this)
        .rfind(&pattern)
        .map(|i| Ok(i.into()))
        .unwrap_or_else(|| Ok((-1).into())) // Not found
}

/// Implements String.localeCompare
/// NOTE: Despite the declaration of this function in the documentation, FP does not support multiple strings in comparison
pub fn locale_compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let other = args.get_string(activation, 0)?;

    for (tc, oc) in this.iter().zip(other.iter()) {
        let res = (tc as i32) - (oc as i32);
        if res != 0 {
            return Ok(Value::Integer(res));
        }
    }

    if this.len() < other.len() {
        return Ok(Value::Integer(-1));
    }

    if this.len() > other.len() {
        return Ok(Value::Integer(1));
    }

    Ok(Value::Integer(0))
}

/// Implements `String.match`. This function can't be named `match` because it's a Rust keyword.
pub fn match_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let pattern = args.get_value(0);

    let regexp_class = activation.avm2().classes().regexp;
    let pattern = if pattern.is_of_type(activation, regexp_class.inner_class_definition()) {
        pattern
    } else {
        let string = pattern.coerce_to_string(activation)?;
        regexp_class.construct(activation, &[Value::String(string)])?
    };

    let pattern = pattern.as_object().unwrap();
    if let Some(mut regexp) = pattern.as_regexp_mut(activation.gc()) {
        let mut storage = ArrayStorage::new(0);
        if regexp.flags().contains(RegExpFlags::GLOBAL) {
            let mut last = regexp.last_index();
            let old_last_index = regexp.last_index();
            regexp.set_last_index(0);
            while let Some(result) = regexp.exec(this) {
                if regexp.last_index() == last {
                    break;
                }
                storage.push(AvmString::new(activation.gc(), &this[result.range()]).into());
                last = regexp.last_index();
            }
            regexp.set_last_index(0);
            if old_last_index == regexp.last_index() {
                regexp.set_last_index(1);
            }

            return Ok(ArrayObject::from_storage(activation, storage).into());
        } else {
            let old = regexp.last_index();
            regexp.set_last_index(0);
            if let Some(result) = regexp.exec(this) {
                let substrings = result.groups().map(|range| &this[range.unwrap_or(0..0)]);

                let mut storage = ArrayStorage::new(0);
                for substring in substrings {
                    storage.push(AvmString::new(activation.gc(), substring).into());
                }
                regexp.set_last_index(old);

                return Ok(ArrayObject::from_storage(activation, storage).into());
            } else {
                regexp.set_last_index(old);
                // If the pattern parameter is a String or a non-global regular expression
                // and no match is found, the method returns null
                return Ok(Value::Null);
            }
        };
    }

    Ok(Value::Null)
}

/// Implements `String.replace`
pub fn replace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;
    let pattern = args.get_value(0);
    let replacement = args.get_value(1);

    if let Some(regexp) = pattern
        .as_object()
        .as_ref()
        .and_then(|o| o.as_regexp_object())
    {
        // Replacement is either a function or treatable as string.
        if let Some(f) = replacement.as_object().and_then(|o| o.as_function_object()) {
            return Ok(RegExp::replace_fn(regexp, activation, this, &f)?.into());
        } else {
            let replacement = replacement.coerce_to_string(activation)?;
            return Ok(RegExp::replace_string(regexp, activation, this, replacement)?.into());
        }
    }

    // Handles patterns which are treatable as string.
    let pattern = pattern.coerce_to_string(activation)?;
    if let Some(position) = this.find(&pattern) {
        let mut ret = WString::from(&this[..position]);
        // Replacement is either a function or treatable as string.
        if let Some(f) = replacement.as_object().and_then(|o| o.as_function_object()) {
            let args = [pattern.into(), position.into(), this.into()];
            let v = f.call(activation, Value::Null, &args)?;
            ret.push_str(v.coerce_to_string(activation)?.as_wstr());
        } else {
            let replacement = replacement.coerce_to_string(activation)?;
            ret.push_str(&replacement);
        }
        ret.push_str(&this[position + pattern.len()..]);

        Ok(AvmString::new(activation.gc(), ret).into())
    } else {
        Ok(this.into())
    }
}

/// Implements `String.search`
pub fn search<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let pattern = args.get_value(0);

    let regexp_class = activation.avm2().classes().regexp;
    let pattern = if pattern.is_of_type(activation, regexp_class.inner_class_definition()) {
        pattern
    } else {
        let string = pattern.coerce_to_string(activation)?;
        regexp_class.construct(activation, &[Value::String(string)])?
    };

    let pattern = pattern.as_object().unwrap();
    if let Some(mut regexp) = pattern.as_regexp_mut(activation.gc()) {
        let old = regexp.last_index();
        regexp.set_last_index(0);
        if let Some(result) = regexp.exec(this) {
            let found_index = result.groups().flatten().next().unwrap_or_default().start as i32;

            regexp.set_last_index(old);
            return Ok(Value::Integer(found_index));
        } else {
            regexp.set_last_index(old);
            // If the pattern parameter is a String or a non-global regular expression
            // and no match is found, the method returns -1
            return Ok(Value::Integer(-1));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.slice`
pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let start_index = args.get_f64(activation, 0)?;
    let start_index = string_wrapping_index(start_index, this.len());

    let end_index = args.get_f64(activation, 1)?;
    let end_index = string_wrapping_index(end_index, this.len());

    if start_index < end_index {
        Ok(activation
            .strings()
            .substring(this, start_index..end_index)
            .into())
    } else {
        Ok(istr!("").into())
    }
}

/// Implements `String.split`
pub fn split<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let delimiter = args.get_value(0);

    let limit = match args.get_value(1) {
        Value::Undefined => usize::MAX,
        limit => limit.coerce_to_u32(activation)? as usize,
    };

    if let Some(mut regexp) = delimiter
        .as_object()
        .as_ref()
        .and_then(|o| o.as_regexp_mut(activation.gc()))
    {
        return Ok(regexp.split(activation, this, limit).into());
    }

    let delimiter = delimiter.coerce_to_string(activation)?;

    let storage = if delimiter.is_empty() {
        // When using an empty delimiter, Str::split adds an extra beginning and trailing item, but Flash does not.
        // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
        // Special case this to match Flash's behavior.
        this.iter()
            .take(limit)
            .map(|c| Value::from(activation.strings().make_char(c)))
            .collect()
    } else {
        this.split(&delimiter)
            .take(limit)
            .map(|c| Value::from(AvmString::new(activation.gc(), c)))
            .collect()
    };

    Ok(ArrayObject::from_storage(activation, storage).into())
}

/// Implements `String.substr`
pub fn substr<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let start_index = args.get_f64(activation, 0)?;
    let start_index = string_wrapping_index(start_index, this.len());

    let len = args.get_f64(activation, 1)?;

    let len = if len.is_nan() {
        0.0
    } else {
        len.min(0x7fffffff as f64)
    };

    let len = if len < 0. {
        if len.is_infinite() {
            0.
        } else if len <= -1.0 {
            let wrapped_around = this.len() as f64 + len;
            if wrapped_around as usize + start_index >= this.len() {
                return Ok(istr!("").into());
            };
            wrapped_around
        } else {
            (len as isize) as f64
        }
    } else {
        len
    };

    let end_index = this.len().min(start_index + len as usize);

    Ok(activation
        .strings()
        .substring(this, start_index..end_index)
        .into())
}

/// Implements `String.substring`
pub fn substring<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    let start_index = args.get_f64(activation, 0)?;
    let mut start_index = string_index(start_index, this.len());

    let end_index = args.get_f64(activation, 1)?;
    let mut end_index = string_index(end_index, this.len());

    if end_index < start_index {
        std::mem::swap(&mut end_index, &mut start_index);
    }

    Ok(activation
        .strings()
        .substring(this, start_index..end_index)
        .into())
}

/// Implements `String.toLowerCase`
pub fn to_lower_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    Ok(AvmString::new(
        activation.gc(),
        this.iter()
            .map(crate::string::utils::swf_to_lowercase)
            .collect::<WString>(),
    )
    .into())
}

/// Implements `String.toUpperCase`
pub fn to_upper_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.coerce_to_string(activation)?;

    Ok(AvmString::new(
        activation.gc(),
        this.iter()
            .map(crate::string::utils::swf_to_uppercase)
            .collect::<WString>(),
    )
    .into())
}

/// Normalizes an  index parameter used in `String` functions such as `substring`.
/// The returned index will be within the range of `[0, len]`.
fn string_index(i: f64, len: usize) -> usize {
    if i == f64::INFINITY {
        len
    } else if i < 0. {
        0
    } else {
        (i as usize).min(len)
    }
}

/// Normalizes an wrapping index parameter used in `String` functions such as `slice`.
/// Values less than or equal to -1.0 will count backwards from `len`.
/// The returned index will be within the range of `[0, len]`.
fn string_wrapping_index(i: f64, len: usize) -> usize {
    if i <= -1.0 {
        if i.is_infinite() {
            return 0;
        }
        let offset = i as isize;
        len.saturating_sub((-offset) as usize)
    } else {
        if i.is_infinite() {
            return len;
        }
        (i as usize).min(len)
    }
}
