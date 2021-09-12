//! `String` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{primitive_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::utils as string_utils;
use crate::string::AvmString;
use crate::avm2::{ArrayObject, ArrayStorage};
use gc_arena::{GcCell, MutationContext};
use std::iter;

/// Implements `String`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if let Some(mut value) = this.as_primitive_mut(activation.context.gc_context) {
            if !matches!(*value, Value::String(_)) {
                *value = args
                    .get(0)
                    .unwrap_or(&Value::String("".into()))
                    .coerce_to_string(activation)?
                    .into();
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `length` property's getter
fn length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            return Ok(s.encode_utf16().count().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.charAt`
fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            // This function takes Number, so if we use coerce_to_i32 instead of coerce_to_number, the value may overflow.
            let n = args
                .get(0)
                .unwrap_or(&Value::Number(0.0))
                .coerce_to_number(activation)?;
            if n < 0.0 {
                return Ok("".into());
            }

            let index = if !n.is_nan() { n as usize } else { 0 };
            let ret = s
                .encode_utf16()
                .nth(index)
                .map(|c| string_utils::utf16_code_unit_to_char(c).to_string())
                .unwrap_or_default();
            return Ok(AvmString::new(activation.context.gc_context, ret).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.charCodeAt`
fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            // This function takes Number, so if we use coerce_to_i32 instead of coerce_to_number, the value may overflow.
            let n = args
                .get(0)
                .unwrap_or(&Value::Number(0.0))
                .coerce_to_number(activation)?;
            if n < 0.0 {
                return Ok(f64::NAN.into());
            }

            let index = if !n.is_nan() { n as usize } else { 0 };
            let ret = s
                .encode_utf16()
                .nth(index)
                .map(f64::from)
                .unwrap_or(f64::NAN);
            return Ok(ret.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.concat`
fn concat<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut ret = Value::from(this).coerce_to_string(activation)?.to_string();
        for arg in args {
            let s = arg.coerce_to_string(activation)?;
            ret.push_str(&s)
        }
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.fromCharCode`
fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let mut out = String::with_capacity(args.len());
    for arg in args {
        let i = arg.coerce_to_u32(activation)? as u16;
        if i == 0 {
            continue;
        }
        out.push(string_utils::utf16_code_unit_to_char(i));
    }
    Ok(AvmString::new(activation.context.gc_context, out).into())
}

/// Implements `String.indexOf`
fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
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

        return if start_index >= this.len() {
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
        };
    }

    Ok(Value::Undefined)
}

/// Implements `String.lastIndexOf`
fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
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

        return if pattern.is_empty() {
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
        };
    }

    Ok(Value::Undefined)
}

/// Implements `String.match`
fn match_s<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let (Some(this), pattern) = (this, args.get(0).unwrap_or(&Value::Undefined)) {
        let this = Value::from(this).coerce_to_string(activation)?;

        let regexp_class = activation.avm2().classes().regexp;
        let pattern = if !pattern.is_of_type(activation, regexp_class)? {
            let string = pattern.coerce_to_string(activation)?;
            regexp_class.construct(activation, &[Value::String(string)])?
        } else {
            pattern.coerce_to_object(activation)?
        };

        if let Some(mut regexp) = pattern.as_regexp_mut(activation.context.gc_context) {
            let mut storage = ArrayStorage::new(0);
            if regexp.global() {
                let mut last = regexp.last_index();
                regexp.set_last_index(0);
                while let Some(result) = regexp.exec(&this) {
                    if regexp.last_index() == last {
                        break;
                    }
                    storage.push(
                        AvmString::new(
                            activation.context.gc_context,
                            this[result.range()].to_string(),
                        )
                        .into(),
                    );
                    last = regexp.last_index();
                }
                regexp.set_last_index(0);
                return Ok(ArrayObject::from_storage(activation, storage)
                    .unwrap()
                    .into());
            } else {
                let old = regexp.last_index();
                regexp.set_last_index(0);
                if let Some(result) = regexp.exec(&this) {
                    let substrings = result
                        .groups()
                        .map(|range| this[range.unwrap_or(0..0)].to_string());

                    let mut storage = ArrayStorage::new(0);
                    for substring in substrings {
                        storage
                            .push(AvmString::new(activation.context.gc_context, substring).into());
                    }
                    regexp.set_last_index(old);
                    return Ok(ArrayObject::from_storage(activation, storage)
                        .unwrap()
                        .into());
                } else {
                    regexp.set_last_index(old);
                    // If the pattern parameter is a String or a non-global regular expression
                    // and no match is found, the method returns null
                    return Ok(Value::Null);
                }
            };
        };
    }

    Ok(Value::Null)
}

/// Implements `String.slice`
fn slice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let this = Value::from(this).coerce_to_string(activation)?;
        let this_len = this.encode_utf16().count();
        let start_index = match args.get(0) {
            None => 0,
            Some(n) => {
                let n = n.coerce_to_number(activation)?;
                string_wrapping_index(n, this_len)
            }
        };
        let end_index = match args.get(1) {
            None => this_len,
            Some(n) => {
                let n = n.coerce_to_number(activation)?;
                string_wrapping_index(n, this_len)
            }
        };
        return if start_index < end_index {
            let ret = string_utils::utf16_iter_to_string(
                this.encode_utf16()
                    .skip(start_index)
                    .take(end_index - start_index),
            );
            Ok(AvmString::new(activation.context.gc_context, ret).into())
        } else {
            Ok("".into())
        };
    }
    Ok(Value::Undefined)
}

/// Implements `String.split`
fn split<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let delimiter = args.get(0).unwrap_or(&Value::Undefined);
        if matches!(delimiter, Value::Undefined) {
            let this = Value::from(this);
            return Ok(
                ArrayObject::from_storage(activation, iter::once(this).collect())
                    .unwrap()
                    .into(),
            );
        }
        if delimiter
            .coerce_to_object(activation)?
            .as_regexp()
            .is_some()
        {
            log::warn!("string.split(regex) - not implemented");
        }
        let this = Value::from(this).coerce_to_string(activation)?;
        let delimiter = delimiter.coerce_to_string(activation)?;
        let limit = match args.get(1).unwrap_or(&Value::Undefined) {
            Value::Undefined => usize::MAX,
            limit => limit.coerce_to_i32(activation)?.max(0) as usize,
        };
        if delimiter.is_empty() {
            // When using an empty delimiter, Rust's str::split adds an extra beginning and trailing item, but Flash does not.
            // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
            // Special case this to match Flash's behavior.
            return Ok(ArrayObject::from_storage(
                activation,
                this.chars()
                    .take(limit)
                    .map(|c| AvmString::new(activation.context.gc_context, c.to_string()))
                    .collect(),
            )
            .unwrap()
            .into());
        } else {
            return Ok(ArrayObject::from_storage(
                activation,
                this.split(delimiter.as_ref())
                    .take(limit)
                    .map(|c| AvmString::new(activation.context.gc_context, c.to_string()))
                    .collect(),
            )
            .unwrap()
            .into());
        }
    }
    Ok(Value::Undefined)
}

/// Implements `String.substr`
fn substr<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;

        if args.is_empty() {
            return Ok(Value::from(this));
        }

        let this_len = this.encode_utf16().count();

        let start_index = string_wrapping_index(
            args.get(0)
                .unwrap_or(&Value::Number(0.))
                .coerce_to_number(activation)?,
            this_len,
        );

        let len = args
            .get(1)
            .unwrap_or(&Value::Number(0x7fffffff as f64))
            .coerce_to_number(activation)?;

        let len = if len == f64::INFINITY {
            this_len
        } else {
            len as usize
        };

        let ret =
            string_utils::utf16_iter_to_string(this.encode_utf16().skip(start_index).take(len));
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.substring`
fn substring<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;

        if args.is_empty() {
            return Ok(Value::from(this));
        }

        let this_len = this.encode_utf16().count();

        let mut start_index = string_index(
            args.get(0)
                .unwrap_or(&Value::Number(0.))
                .coerce_to_number(activation)?,
            this_len,
        );

        let mut end_index = string_index(
            args.get(1)
                .unwrap_or(&Value::Number(0x7fffffff as f64))
                .coerce_to_number(activation)?,
            this_len,
        );

        if end_index < start_index {
            std::mem::swap(&mut end_index, &mut start_index);
        }

        let ret = string_utils::utf16_iter_to_string(
            this.encode_utf16()
                .skip(start_index)
                .take(end_index - start_index),
        );
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Construct `String`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "String"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<String instance initializer>", mc),
        Method::from_builtin(class_init, "<String class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    write.set_instance_allocator(primitive_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("length", Some(length), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("charAt", char_at),
        ("charCodeAt", char_code_at),
        ("concat", concat),
        ("indexOf", index_of),
        ("lastIndexOf", last_index_of),
        ("match", match_s),
        ("slice", slice),
        ("split", split),
        ("substr", substr),
        ("substring", substring),
    ];
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("fromCharCode", from_char_code)];
    write.define_as3_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    class
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
/// Negative values will count backwards from `len`.
/// The returned index will be within the range of `[0, len]`.
fn string_wrapping_index(i: f64, len: usize) -> usize {
    if i < 0. {
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
