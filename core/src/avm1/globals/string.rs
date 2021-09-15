//! `String` class impl
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::string::{utils as string_utils, AvmString, WString};
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
    let this_val = Value::from(this);
    let string = this_val.coerce_to_string(activation)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    let ret = usize::try_from(i)
        .ok()
        .and_then(|i| string.try_get(i))
        .map(WString::from_unit)
        .map(|ret| AvmString::new_ucs2(activation.context.gc_context, ret))
        .unwrap_or_else(|| "".into());

    Ok(ret.into())
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
        this.try_get(i as usize).map(f64::from).unwrap_or(f64::NAN)
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
    let mut ret: WString = Value::from(this)
        .coerce_to_string(activation)?
        .as_ucs2()
        .into();
    for arg in args {
        let s = arg.coerce_to_string(activation)?;
        ret.push_str(s.as_ucs2())
    }
    Ok(AvmString::new_ucs2(activation.context.gc_context, ret).into())
}

fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
    Ok(AvmString::new_ucs2(activation.context.gc_context, out).into())
}

fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

    this.try_slice(start_index..)
        .and_then(|s| s.find(pattern.as_ucs2()))
        .map(|i| Ok((i + start_index).into()))
        .unwrap_or_else(|| Ok((-1).into())) // Out of range or not found
}

fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

    this.try_slice(..start_index)
        .unwrap_or_else(|| this.as_ucs2())
        .rfind(pattern.as_ucs2())
        .map(|i| Ok(i.into()))
        .unwrap_or_else(|| Ok((-1).into())) // Not found
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
        let ret = WString::from(this.slice(start_index..end_index));
        Ok(AvmString::new_ucs2(activation.context.gc_context, ret).into())
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
        // When using an empty delimiter, Str::split adds an extra beginning and trailing item, but Flash does not.
        // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
        // Special case this to match Flash's behavior.
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            this.iter().take(limit).map(|c| {
                AvmString::new_ucs2(activation.context.gc_context, WString::from_unit(c)).into()
            }),
        )
        .into())
    } else {
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            this.split(delimiter.as_ucs2())
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
        let ret = WString::from(this.slice(start_index..end_index));
        Ok(AvmString::new_ucs2(activation.context.gc_context, ret).into())
    } else {
        Ok("".into())
    }
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
    let mut start_index = string_index(args.get(0).unwrap().coerce_to_i32(activation)?, this.len());

    let mut end_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => string_index(n.coerce_to_i32(activation)?, this.len()),
    };

    // substring automatically swaps the start/end if they are flipped.
    if end_index < start_index {
        std::mem::swap(&mut end_index, &mut start_index);
    }
    let ret = WString::from(this.slice(start_index..end_index));
    Ok(AvmString::new_ucs2(activation.context.gc_context, ret).into())
}

fn to_lower_case<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_val = Value::from(this);
    let this = this_val.coerce_to_string(activation)?;
    Ok(AvmString::new_ucs2(
        activation.context.gc_context,
        this.iter()
            .map(string_utils::swf_to_lowercase)
            .collect::<WString>(),
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
    Ok(AvmString::new_ucs2(
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
