//! `String` class impl

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::value_object::ValueObject;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use crate::string_utils;
use enumset::EnumSet;
use gc_arena::MutationContext;

/// `String` constructor
pub fn string<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let value = match args.get(0).cloned() {
        Some(Value::String(s)) => s,
        Some(o) => o.coerce_to_string(avm, ac)?,
        _ => String::new(),
    };

    if let Some(mut vbox) = this.as_value_object() {
        let len = value.encode_utf16().count();
        vbox.set_length(ac.gc_context, len);
        vbox.replace_value(ac.gc_context, value.clone().into());
    }

    Ok(value.into())
}

pub fn create_string_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    string_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let string = FunctionObject::function(
        gc_context,
        Executable::Native(string),
        fn_proto,
        string_proto,
    );
    let mut object = string.as_script_object().unwrap();

    object.force_set_function(
        "fromCharCode",
        from_char_code,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    string
}

/// Creates `String.prototype`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string_proto = ValueObject::empty_box(gc_context, Some(proto));
    let mut object = string_proto.as_script_object().unwrap();

    object.force_set_function(
        "toString",
        to_string_value_of,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "valueOf",
        to_string_value_of,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "charAt",
        char_at,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "charCodeAt",
        char_code_at,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "concat",
        concat,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "indexOf",
        index_of,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "lastIndexOf",
        last_index_of,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "slice",
        slice,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "split",
        split,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "substr",
        substr,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "substring",
        substring,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "toLowerCase",
        to_lower_case,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.force_set_function(
        "toUpperCase",
        to_upper_case,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    string_proto
}

fn char_at<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // TODO: Will return REPLACEMENT_CHAR if this indexes a character outside the BMP, losing info about the surrogate.
    // When we improve our string representation, the unpaired surrogate should be returned.
    let this = Value::from(this).coerce_to_string(avm, context)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(avm, context)?;
    let ret = if i >= 0 {
        this.encode_utf16()
            .nth(i as usize)
            .map(|c| utf16_code_unit_to_char(c).to_string())
            .unwrap_or_default()
    } else {
        "".into()
    };
    Ok(ret.into())
}

fn char_code_at<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this).coerce_to_string(avm, context)?;
    let i = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(avm, context)?;
    let ret = if i > 0 {
        this.encode_utf16()
            .nth(i as usize)
            .map(f64::from)
            .unwrap_or(std::f64::NAN)
    } else {
        std::f64::NAN
    };
    Ok(ret.into())
}

fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut ret = Value::from(this).coerce_to_string(avm, context)?;
    for arg in args {
        let s = arg.clone().coerce_to_string(avm, context)?;
        ret.push_str(&s)
    }
    Ok(ret.into())
}

fn from_char_code<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // TODO: Unpaired surrogates will be replace with Unicode replacement char.
    let mut out = String::with_capacity(args.len());
    for arg in args {
        let i = arg.coerce_to_u16(avm, context)?;
        if i == 0 {
            // Stop at a null-terminator.
            break;
        }
        out.push(utf16_code_unit_to_char(i));
    }
    Ok(out.into())
}

fn index_of<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this)
        .coerce_to_string(avm, context)?
        .encode_utf16()
        .collect::<Vec<u16>>();
    let pattern = match args.get(0) {
        None | Some(Value::Undefined) => return Ok(Value::Undefined.into()),
        Some(s) => s
            .clone()
            .coerce_to_string(avm, context)?
            .encode_utf16()
            .collect::<Vec<_>>(),
    };
    let start_index = {
        let n = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(avm, context)?;
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
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this)
        .coerce_to_string(avm, context)?
        .encode_utf16()
        .collect::<Vec<u16>>();
    let pattern = match args.get(0) {
        None | Some(Value::Undefined) => return Ok(Value::Undefined.into()),
        Some(s) => s
            .clone()
            .coerce_to_string(avm, context)?
            .encode_utf16()
            .collect::<Vec<_>>(),
    };
    let start_index = match args.get(1) {
        None | Some(Value::Undefined) => this.len(),
        Some(n) => {
            let n = n.coerce_to_i32(avm, context)?;
            if n >= 0 {
                let n = n as usize;
                if n <= this.len() {
                    n
                } else {
                    this.len()
                }
            } else {
                0
            }
        }
    };

    if pattern.is_empty() {
        // Empty pattern is found immediately.
        Ok((start_index as f64).into())
    } else if let Some((i, _)) = this[..]
        .windows(pattern.len())
        .enumerate()
        .take(start_index)
        .rev()
        .find(|(_, w)| *w == &pattern[..])
    {
        Ok((i as f64).into())
    } else {
        // Not found
        Ok((-1).into())
    }
}

fn slice<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        // No args returns undefined immediately.
        return Ok(Value::Undefined.into());
    }

    let this = Value::from(this).coerce_to_string(avm, context)?;
    let this_len = this.encode_utf16().count();
    let start_index = string_wrapping_index(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(avm, context)?,
        this_len,
    );
    let end_index = match args.get(1) {
        None | Some(Value::Undefined) => this_len,
        Some(n) => string_wrapping_index(n.coerce_to_i32(avm, context)?, this_len),
    };
    if start_index < end_index {
        let ret = utf16_iter_to_string(
            this.encode_utf16()
                .skip(start_index)
                .take(end_index - start_index),
        );
        Ok(ret.into())
    } else {
        Ok("".into())
    }
}

fn split<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this).coerce_to_string(avm, context)?;
    let delimiter = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .clone()
        .coerce_to_string(avm, context)?;
    let limit = match args.get(1) {
        None | Some(Value::Undefined) => std::usize::MAX,
        Some(n) => std::cmp::max(0, n.coerce_to_i32(avm, context)?) as usize,
    };
    let array = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));
    if !delimiter.is_empty() {
        for (i, token) in this.split(&delimiter).take(limit).enumerate() {
            array.set_array_element(i, token.to_string().into(), context.gc_context);
        }
    } else {
        // When using an empty "" delimiter, Rust's str::split adds an extra beginning and trailing item, but Flash does not.
        // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
        // Special case this to match Flash's behavior.
        for (i, token) in this.chars().take(limit).enumerate() {
            array.set_array_element(i, token.to_string().into(), context.gc_context);
        }
    }
    Ok(array.into())
}

fn substr<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        return Ok(Value::Undefined.into());
    }

    let this = Value::from(this).coerce_to_string(avm, context)?;
    let this_len = this.encode_utf16().count();
    let start_index =
        string_wrapping_index(args.get(0).unwrap().coerce_to_i32(avm, context)?, this_len);

    let len = match args.get(1) {
        None | Some(Value::Undefined) => this_len,
        Some(n) => string_index(n.coerce_to_i32(avm, context)?, this_len),
    };

    let ret = utf16_iter_to_string(this.encode_utf16().skip(start_index).take(len));
    Ok(ret.into())
}

fn substring<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        return Ok(Value::Undefined.into());
    }

    let this = Value::from(this).coerce_to_string(avm, context)?;
    let this_len = this.encode_utf16().count();
    let mut start_index = string_index(args.get(0).unwrap().coerce_to_i32(avm, context)?, this_len);

    let mut end_index = match args.get(1) {
        None | Some(Value::Undefined) => this_len,
        Some(n) => string_index(n.coerce_to_i32(avm, context)?, this_len),
    };

    // substring automatically swaps the start/end if they are flipped.
    if end_index < start_index {
        std::mem::swap(&mut end_index, &mut start_index);
    }
    let ret = utf16_iter_to_string(
        this.encode_utf16()
            .skip(start_index)
            .take(end_index - start_index),
    );
    Ok(ret.into())
}

fn to_lower_case<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this).coerce_to_string(avm, context)?;
    Ok(this
        .chars()
        .map(string_utils::swf_char_to_lowercase)
        .collect::<String>()
        .into())
}

/// `String.toString` / `String.valueOf` impl
pub fn to_string_value_of<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(vbox) = this.as_value_object() {
        if let Value::String(s) = vbox.unbox() {
            return Ok(s.into());
        }
    }

    //TODO: This normally falls back to `[object Object]` or `[type Function]`,
    //implying that `toString` and `valueOf` are inherent object properties and
    //not just methods.
    Ok(Value::Undefined.into())
}

fn to_upper_case<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = Value::from(this).coerce_to_string(avm, context)?;
    Ok(this
        .chars()
        .map(string_utils::swf_char_to_uppercase)
        .collect::<String>()
        .into())
}

/// Normalizes an  index paramter used in `String` functions such as `substring`.
/// The returned index will be within the range of `[0, len]`.
fn string_index(i: i32, len: usize) -> usize {
    if i > 0 {
        let i = i as usize;
        if i < len {
            i
        } else {
            len
        }
    } else {
        0
    }
}

/// Normalizes an wrapping index paramter used in `String` functions such as `slice`.
/// Negative values will count backwards from `len`.
/// The returned index will be within the range of `[0, len]`.
fn string_wrapping_index(i: i32, len: usize) -> usize {
    if i >= 0 {
        let i = i as usize;
        if i < len {
            i
        } else {
            len
        }
    } else {
        let i = (-i) as usize;
        if i <= len {
            len - i
        } else {
            len
        }
    }
}

/// Creates a `String` from an iterator of UTF-16 code units.
/// TODO: Unpaired surrogates will get replaced with the Unicode replacement character.
fn utf16_iter_to_string<I: Iterator<Item = u16>>(it: I) -> String {
    use std::char;
    char::decode_utf16(it)
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// Maps a UTF-16 code unit into a `char`.
/// TODO: Surrogate characters will get replaced with the Unicode replacement character.
fn utf16_code_unit_to_char(c: u16) -> char {
    use std::char;
    char::decode_utf16(std::iter::once(c))
        .next()
        .unwrap()
        .unwrap_or(char::REPLACEMENT_CHARACTER)
}
