//! `String` class impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::string::{utils as string_utils, AvmString, StringContext, WString};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string_value_of; DONT_ENUM | DONT_DELETE);
    "valueOf" => method(to_string_value_of; DONT_ENUM | DONT_DELETE);
    "charAt" => method(char_at; DONT_ENUM | DONT_DELETE);
    "charCodeAt" => method(char_code_at; DONT_ENUM | DONT_DELETE);
    "concat" => method(concat; DONT_ENUM | DONT_DELETE);
    "indexOf" => method(index_of; DONT_ENUM | DONT_DELETE);
    "lastIndexOf" => method(last_index_of; DONT_ENUM | DONT_DELETE);
    "slice" => method(slice; DONT_ENUM | DONT_DELETE);
    "split" => method(split; DONT_ENUM | DONT_DELETE);
    "substr" => method(substr; DONT_ENUM | DONT_DELETE);
    "substring" => method(substring; DONT_ENUM | DONT_DELETE);
    "toLowerCase" => method(to_lower_case; DONT_ENUM | DONT_DELETE);
    "toUpperCase" => method(to_upper_case; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "fromCharCode" => method(from_char_code; DONT_ENUM | DONT_DELETE);
};

/// `String` constructor
pub fn string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = match args.get(0).cloned() {
        Some(Value::String(s)) => s,
        Some(v) => v.coerce_to_string(activation)?,
        _ => AvmString::default(),
    };

    if let Some(mut vbox) = this.as_value_object() {
        let len = value.len();
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
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = match args.get(0).cloned() {
        Some(Value::String(s)) => s,
        Some(v) => v.coerce_to_string(activation)?,
        _ => AvmString::new_utf8(activation.context.gc_context, String::new()),
    };

    Ok(value.into())
}

pub fn create_string_object<'gc>(
    context: &mut StringContext<'gc>,
    string_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(string),
        Executable::Native(string_function),
        fn_proto,
        string_proto,
    );
    let object = string.raw_script_object();
    define_properties_on(OBJECT_DECLS, context, object, fn_proto);
    string
}

/// Creates `String.prototype`.
pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string_proto = ValueObject::empty_box(context.gc_context, proto);
    let object = string_proto.raw_script_object();
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    string_proto
}

fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let string = this_val.coerce_to_string(activation)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    let ret = usize::try_from(i)
        .ok()
        .and_then(|i| string.get(i))
        .map(WString::from_unit)
        .map(|ret| AvmString::new(activation.context.gc_context, ret))
        .unwrap_or_else(|| "".into());

    Ok(ret.into())
}

fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;
    let is_swf5 = activation.swf_version() == 5;
    let ret = if i >= 0 {
        this.get(i as usize)
            .map(f64::from)
            .unwrap_or(if is_swf5 { 0.into() } else { f64::NAN })
    } else {
        f64::NAN
    };
    Ok(ret.into())
}

fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut ret: WString = Value::from(this)
        .coerce_to_string(activation)?
        .as_wstr()
        .into();
    for arg in args {
        let s = arg.coerce_to_string(activation)?;
        ret.push_str(&s);
    }
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut out = WString::with_capacity(args.len(), false);
    for arg in args {
        let i = arg.coerce_to_u16(activation)?;
        if i == 0 {
            // Stop at a null-terminator.
            break;
        }
        out.push(i);
    }
    Ok(AvmString::new(activation.context.gc_context, out).into())
}

fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let pattern = match args.get(0) {
        None => return Ok(Value::Undefined),
        Some(s) => s.clone().coerce_to_string(activation)?,
    };

    let start_index = match args.get(1) {
        None | Some(Value::Undefined) => 0,
        Some(n) => n.coerce_to_i32(activation)?.max(0) as usize,
    };

    this.slice(start_index..)
        .and_then(|s| s.find(&pattern))
        .map(|i| Ok((i + start_index).into()))
        .unwrap_or_else(|| Ok((-1).into())) // Out of range or not found
}

fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let pattern = match args.get(0) {
        None => return Ok(Value::Undefined),
        Some(s) => s.clone().coerce_to_string(activation)?,
    };

    let start_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => match usize::try_from(n.coerce_to_i32(activation)?) {
            Ok(n) => n + pattern.len(),
            Err(_) => return Ok((-1).into()), // Bail out on negative indices.
        },
    };

    this.slice(..start_index)
        .unwrap_or(&this)
        .rfind(&pattern)
        .map(|i| Ok(i.into()))
        .unwrap_or_else(|| Ok((-1).into())) // Not found
}

fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        // No args returns undefined immediately.
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let start_index = string_wrapping_index(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?,
        this.len(),
    );
    let end_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => string_wrapping_index(n.coerce_to_i32(activation)?, this.len()),
    };
    if start_index < end_index {
        let ret = WString::from(&this[start_index..end_index]);
        Ok(AvmString::new(activation.context.gc_context, ret).into())
    } else {
        Ok("".into())
    }
}

fn split<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let limit = match args.get(1).unwrap_or(&Value::Undefined) {
        Value::Undefined => usize::MAX,
        limit => limit.coerce_to_i32(activation)?.max(0) as usize,
    };
    // Contrary to the docs, in SWFv5 the default delimiter is comma (,)
    // and the empty string behaves the same as undefined does in later SWF versions.
    let is_swf5 = activation.swf_version() == 5;
    if let Some(delimiter) = match args.get(0).unwrap_or(&Value::Undefined) {
        &Value::Undefined => is_swf5.then_some(",".into()),
        v => Some(v.coerce_to_string(activation)?).filter(|s| !(is_swf5 && s.is_empty())),
    } {
        if delimiter.is_empty() {
            // When using an empty delimiter, Str::split adds an extra beginning and trailing item,
            // but Flash does not.
            // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
            // Special case this to match Flash's behavior.
            Ok(ArrayObject::new(
                activation.context.gc_context,
                activation.context.avm1.prototypes().array,
                this.iter().take(limit).map(|c| {
                    AvmString::new(activation.context.gc_context, WString::from_unit(c)).into()
                }),
            )
            .into())
        } else {
            Ok(ArrayObject::new(
                activation.context.gc_context,
                activation.context.avm1.prototypes().array,
                this.split(&delimiter)
                    .take(limit)
                    .map(|c| AvmString::new(activation.context.gc_context, c).into()),
            )
            .into())
        }
    } else {
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            [this.into()],
        )
        .into())
    }
}

fn substr<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let start_index = string_wrapping_index(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?,
        this.len(),
    );

    let len = match args.get(1) {
        None | Some(Value::Undefined) => this.len() as i32,
        Some(n) => n.coerce_to_i32(activation)?,
    };
    let end_index = string_wrapping_index((start_index as i32) + len, this.len());

    if start_index < end_index {
        let ret = WString::from(&this[start_index..end_index]);
        Ok(AvmString::new(activation.context.gc_context, ret).into())
    } else {
        Ok("".into())
    }
}

fn substring<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    let mut start_index = string_index(args.get(0).unwrap().coerce_to_i32(activation)?, this.len());

    let mut end_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => string_index(n.coerce_to_i32(activation)?, this.len()),
    };

    // substring automatically swaps the start/end if they are flipped.
    if end_index < start_index {
        std::mem::swap(&mut end_index, &mut start_index);
    }
    let ret = WString::from(&this[start_index..end_index]);
    Ok(AvmString::new(activation.context.gc_context, ret).into())
}

fn to_lower_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    Ok(AvmString::new(
        activation.context.gc_context,
        this.iter()
            .map(string_utils::swf_to_lowercase)
            .collect::<WString>(),
    )
    .into())
}

/// `String.toString` / `String.valueOf` impl
pub fn to_string_value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    Ok(AvmString::new(
        activation.context.gc_context,
        this.iter()
            .map(string_utils::swf_to_uppercase)
            .collect::<WString>(),
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
