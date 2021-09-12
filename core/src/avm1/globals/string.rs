//! `String` class impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::string::{utils as string_utils, AvmString};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string_value_of);
    "valueOf" => method(to_string_value_of);
    "charAt" => method(char_at; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "charCodeAt" => method(char_code_at; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "concat" => method(concat; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "indexOf" => method(index_of; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "lastIndexOf" => method(last_index_of; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "slice" => method(slice; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "split" => method(split; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "substr" => method(substr; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "substring" => method(substring; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "toLowerCase" => method(to_lower_case; DONT_DELETE | DONT_ENUM | READ_ONLY);
    "toUpperCase" => method(to_upper_case; DONT_DELETE | DONT_ENUM | READ_ONLY);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "fromCharCode" => method(from_char_code; DONT_DELETE | DONT_ENUM | READ_ONLY);
};

/// `String` constructor
pub fn string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = match args.get(0).cloned() {
        Some(Value::String(s)) => s,
        Some(v) => v.coerce_to_string(activation)?,
        _ => AvmString::new(activation.context.gc_context, String::new()),
    };

    if let Some(mut vbox) = this.as_value_object() {
        let len = value.encode_utf16().count();
        vbox.define_value(
            activation.context.gc_context,
            "length",
            len.into(),
            Attribute::empty(),
        );
        vbox.replace_value(activation.context.gc_context, value.into());
    }

    Ok(this.into())
}

/// `String` function
pub fn string_function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = match args.get(0).cloned() {
        Some(Value::String(s)) => s,
        Some(v) => v.coerce_to_string(activation)?,
        _ => AvmString::new(activation.context.gc_context, String::new()),
    };

    Ok(value.into())
}

pub fn create_string_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    string_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string = FunctionObject::constructor(
        gc_context,
        Executable::Native(string),
        Executable::Native(string_function),
        Some(fn_proto),
        string_proto,
    );
    let object = string.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    string
}

/// Creates `String.prototype`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string_proto = ValueObject::empty_box(gc_context, Some(proto));
    let object = string_proto.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    string_proto
}

fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Will return REPLACEMENT_CHAR if this indexes a character outside the BMP, losing info about the surrogate.
    // When we improve our string representation, the unpaired surrogate should be returned.
    let this_val = Value::from(this);
    let string = this_val.coerce_to_string(activation)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    let ret = if i >= 0 {
        string
            .encode_utf16()
            .nth(i as usize)
            .map(|c| string_utils::utf16_code_unit_to_char(c).to_string())
            .unwrap_or_default()
    } else {
        "".into()
    };
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    let ret = if i >= 0 {
        this.encode_utf16()
            .nth(i as usize)
            .map(f64::from)
            .unwrap_or(f64::NAN)
    } else {
        f64::NAN
    };
    Ok(ret.into())
}

fn concat<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut ret = Value::from(this).coerce_to_string(activation)?.to_string();
    for arg in args {
        let s = arg.coerce_to_string(activation)?;
        ret.push_str(&s)
    }
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Unpaired surrogates will be replace with Unicode replacement char.
    let mut out = String::with_capacity(args.len());
    for arg in args {
        let i = arg.coerce_to_u16(activation)?;
        if i == 0 {
            // Stop at a null-terminator.
            break;
        }
        out.push(string_utils::utf16_code_unit_to_char(i));
    }
    Ok(AvmString::new(activation.context.gc_context, out).into())
}

fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this)
        .coerce_to_string(activation)?
        .encode_utf16()
        .collect::<Vec<u16>>();
    let pattern = match args.get(0) {
        None => return Ok(Value::Undefined),
        Some(s) => s
            .clone()
            .coerce_to_string(activation)?
            .encode_utf16()
            .collect::<Vec<_>>(),
    };
    let start_index = {
        let n = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        if n >= 0 {
            n as usize
        } else {
            0
        }
    };

    if start_index >= this.len() {
        // Out of range
        Ok((-1).into())
    } else if pattern.is_empty() {
        // Empty pattern is found immediately.
        Ok((start_index as f64).into())
    } else if let Some(mut pos) = this[start_index..]
        .windows(pattern.len())
        .position(|w| w == &pattern[..])
    {
        pos += start_index;
        Ok((pos as f64).into())
    } else {
        // Not found
        Ok((-1).into())
    }
}

fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this)
        .coerce_to_string(activation)?
        .encode_utf16()
        .collect::<Vec<u16>>();
    let pattern = match args.get(0) {
        None => return Ok(Value::Undefined),
        Some(s) => s
            .clone()
            .coerce_to_string(activation)?
            .encode_utf16()
            .collect::<Vec<_>>(),
    };
    let start_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => n.coerce_to_i32(activation)?.max(0) as usize,
    };

    if pattern.is_empty() {
        // Empty pattern is found immediately.
        Ok(start_index.into())
    } else if let Some((i, _)) = this[..]
        .windows(pattern.len())
        .enumerate()
        .take(start_index + 1)
        .rev()
        .find(|(_, w)| *w == &pattern[..])
    {
        Ok(i.into())
    } else {
        // Not found
        Ok((-1).into())
    }
}

fn slice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        // No args returns undefined immediately.
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let this_len = this.encode_utf16().count();
    let start_index = string_wrapping_index(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?,
        this_len,
    );
    let end_index = match args.get(1) {
        None | Some(Value::Undefined) => this_len,
        Some(n) => string_wrapping_index(n.coerce_to_i32(activation)?, this_len),
    };
    if start_index < end_index {
        let ret = string_utils::utf16_iter_to_string(
            this.encode_utf16()
                .skip(start_index)
                .take(end_index - start_index),
        );
        Ok(AvmString::new(activation.context.gc_context, ret).into())
    } else {
        Ok("".into())
    }
}

fn split<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let delimiter = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let limit = match args.get(1).unwrap_or(&Value::Undefined) {
        Value::Undefined => usize::MAX,
        limit => limit.coerce_to_i32(activation)?.max(0) as usize,
    };
    if delimiter.is_empty() {
        // When using an empty delimiter, Rust's str::split adds an extra beginning and trailing item, but Flash does not.
        // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
        // Special case this to match Flash's behavior.
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            this.chars()
                .take(limit)
                .map(|c| AvmString::new(activation.context.gc_context, c.to_string()).into()),
        )
        .into())
    } else {
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            this.split(delimiter.as_ref())
                .take(limit)
                .map(|c| AvmString::new(activation.context.gc_context, c.to_string()).into()),
        )
        .into())
    }
}

fn substr<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let this_len = this.encode_utf16().count();
    let start_index = string_wrapping_index(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?,
        this_len,
    );

    let len = match args.get(1) {
        None | Some(Value::Undefined) => this_len as i32,
        Some(n) => n.coerce_to_i32(activation)?,
    };
    let end_index = string_wrapping_index((start_index as i32) + len, this_len);
    let len = end_index.saturating_sub(start_index);

    let ret = string_utils::utf16_iter_to_string(this.encode_utf16().skip(start_index).take(len));
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn substring<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let this_len = this.encode_utf16().count();
    let mut start_index = string_index(args.get(0).unwrap().coerce_to_i32(activation)?, this_len);

    let mut end_index = match args.get(1) {
        None | Some(Value::Undefined) => this_len,
        Some(n) => string_index(n.coerce_to_i32(activation)?, this_len),
    };

    // substring automatically swaps the start/end if they are flipped.
    if end_index < start_index {
        std::mem::swap(&mut end_index, &mut start_index);
    }
    let ret = string_utils::utf16_iter_to_string(
        this.encode_utf16()
            .skip(start_index)
            .take(end_index - start_index),
    );
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn to_lower_case<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    Ok(AvmString::new(
        activation.context.gc_context,
        this.chars()
            .map(string_utils::swf_char_to_lowercase)
            .collect::<String>(),
    )
    .into())
}

/// `String.toString` / `String.valueOf` impl
pub fn to_string_value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vbox) = this.as_value_object() {
        if let Value::String(s) = vbox.unbox() {
            return Ok(s.into());
        }
    }

    //TODO: This normally falls back to `[object Object]` or `[type Function]`,
    //implying that `toString` and `valueOf` are inherent object properties and
    //not just methods.
    Ok(Value::Undefined)
}

fn to_upper_case<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    Ok(AvmString::new(
        activation.context.gc_context,
        this.chars()
            .map(string_utils::swf_char_to_uppercase)
            .collect::<String>(),
    )
    .into())
}

/// Normalizes an  index parameter used in `String` functions such as `substring`.
/// The returned index will be within the range of `[0, len]`.
fn string_index(i: i32, len: usize) -> usize {
    if i < 0 {
        0
    } else {
        (i as usize).min(len)
    }
}

/// Normalizes an wrapping index parameter used in `String` functions such as `slice`.
/// Negative values will count backwards from `len`.
/// The returned index will be within the range of `[0, len]`.
fn string_wrapping_index(i: i32, len: usize) -> usize {
    if i < 0 {
        let offset = i as isize;
        len.saturating_sub((-offset) as usize)
    } else {
        (i as usize).min(len)
    }
}
