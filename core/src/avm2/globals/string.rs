//! `String` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::regexp::RegExpFlags;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use crate::avm2::{ArrayObject, ArrayStorage};
use crate::string::{AvmString, WString};
use gc_arena::GcCell;
use std::iter;

// All of these methods will be defined as both
// AS3 instance methods and methods on the `String` class prototype.
const PUBLIC_INSTANCE_AND_PROTO_METHODS: &[(&str, NativeMethodImpl)] = &[
    ("toUpperCase", to_upper_case),
    ("charCodeAt", char_code_at),
    ("search", search),
    ("concat", concat),
    ("slice", slice),
    ("match", match_s),
    ("valueOf", value_of),
    ("charAt", char_at),
    ("substr", substr),
    ("toString", to_string),
    ("toLocaleLowerCase", to_lower_case),
    ("indexOf", index_of),
    ("replace", replace),
    ("split", split),
    ("substring", substring),
    ("lastIndexOf", last_index_of),
    ("toLocaleUpperCase", to_upper_case),
    ("localeCompare", locale_compare),
    ("toLowerCase", to_lower_case),
];

/// Implements `String`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let scope = activation.create_scopechain();
        let gc_context = activation.context.gc_context;
        let this_class = this.as_class_object().unwrap();
        let proto = this_class.prototype();

        for (name, method) in PUBLIC_INSTANCE_AND_PROTO_METHODS {
            proto.set_string_property_local(
                *name,
                FunctionObject::from_method(
                    activation,
                    Method::from_builtin(*method, name, gc_context),
                    scope,
                    None,
                    Some(this_class),
                )
                .into(),
                activation,
            )?;
            proto.set_local_property_is_enumerable(gc_context, (*name).into(), false);
        }
    }
    Ok(Value::Undefined)
}

/// Implements `length` property's getter
fn length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            return Ok(s.len().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.charAt`
fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
                .get(index)
                .map(WString::from_unit)
                .map(|s| AvmString::new(activation.context.gc_context, s))
                .unwrap_or_default();
            return Ok(ret.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.charCodeAt`
fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
            let ret = s.get(index).map(f64::from).unwrap_or(f64::NAN);
            return Ok(ret.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.concat`
fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let mut ret = WString::from(Value::from(this).coerce_to_string(activation)?.as_wstr());
        for arg in args {
            let s = arg.coerce_to_string(activation)?;
            ret.push_str(&s);
        }
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.fromCharCode`
fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut out = WString::with_capacity(args.len(), false);
    for arg in args {
        let i = arg.coerce_to_u32(activation)? as u16;
        if i == 0 {
            // Ignore nulls.
            continue;
        }
        out.push(i);
    }
    Ok(AvmString::new(activation.context.gc_context, out).into())
}

/// Implements `String.indexOf`
fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this = Value::from(this).coerce_to_string(activation)?;
        let pattern = match args.get(0) {
            None => return Ok(Value::Undefined),
            Some(s) => s.clone().coerce_to_string(activation)?,
        };

        let start_index = match args.get(1) {
            None | Some(Value::Undefined) => 0,
            Some(n) => n.coerce_to_i32(activation)?.max(0) as usize,
        };

        return this
            .slice(start_index..)
            .and_then(|s| s.find(&pattern))
            .map(|i| Ok((i + start_index).into()))
            .unwrap_or_else(|| Ok((-1).into())); // Out of range or not found
    }

    Ok(Value::Undefined)
}

/// Implements `String.lastIndexOf`
fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
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

        return this
            .slice(..start_index)
            .unwrap_or(&this)
            .rfind(&pattern)
            .map(|i| Ok(i.into()))
            .unwrap_or_else(|| Ok((-1).into())); // Not found
    }

    Ok(Value::Undefined)
}

/// Implements String.localeCompare
/// NOTE: Despite the declaration of this function in the documentation, FP does not support multiple strings in comparison
fn locale_compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this = Value::from(this).coerce_to_string(activation)?;
        let other = match args.get(0) {
            None => Value::Undefined.coerce_to_string(activation)?,
            Some(s) => s.clone().coerce_to_string(activation)?,
        };

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

        return Ok(Value::Integer(0));
    }

    Ok(Value::Undefined)
}

/// Implements `String.match`
fn match_s<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(this), pattern) = (this, args.get(0).unwrap_or(&Value::Undefined)) {
        let this = Value::from(this).coerce_to_string(activation)?;

        let regexp_class = activation.avm2().classes().regexp;
        let pattern = if !pattern.is_of_type(activation, regexp_class) {
            let string = pattern.coerce_to_string(activation)?;
            regexp_class.construct(activation, &[Value::String(string)])?
        } else {
            pattern.coerce_to_object(activation)?
        };

        if let Some(mut regexp) = pattern.as_regexp_mut(activation.context.gc_context) {
            let mut storage = ArrayStorage::new(0);
            if regexp.flags().contains(RegExpFlags::GLOBAL) {
                let mut last = regexp.last_index();
                let old_last_index = regexp.last_index();
                regexp.set_last_index(0);
                while let Some(result) = regexp.exec(this) {
                    if regexp.last_index() == last {
                        break;
                    }
                    storage.push(
                        AvmString::new(activation.context.gc_context, &this[result.range()]).into(),
                    );
                    last = regexp.last_index();
                }
                regexp.set_last_index(0);
                if old_last_index == regexp.last_index() {
                    regexp.set_last_index(1);
                }
                return Ok(ArrayObject::from_storage(activation, storage)
                    .unwrap()
                    .into());
            } else {
                let old = regexp.last_index();
                regexp.set_last_index(0);
                if let Some(result) = regexp.exec(this) {
                    let substrings = result.groups().map(|range| &this[range.unwrap_or(0..0)]);

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

/// Implements `String.replace`
fn replace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this = Value::from(this).coerce_to_string(activation)?;
        let pattern = args.get(0).unwrap_or(&Value::Undefined);
        let replacement = args.get(1).unwrap_or(&Value::Undefined);
        // Handles regex patterns.
        if let Some(mut regexp) = pattern
            .as_object()
            .as_ref()
            .and_then(|o| o.as_regexp_mut(activation.context.gc_context))
        {
            // Replacement is either a function or treatable as string.
            if let Some(f) = replacement.as_object().and_then(|o| o.as_function_object()) {
                return Ok(regexp.replace_fn(activation, this, &f)?.into());
            } else {
                let replacement = replacement.coerce_to_string(activation)?;
                return Ok(regexp.replace_string(activation, this, replacement)?.into());
            }
        }
        // Handles patterns which are treatable as string.
        let pattern = pattern.coerce_to_string(activation)?;
        if let Some(position) = this.find(&pattern) {
            let mut ret = WString::from(&this[..position]);
            // Replacement is either a function or treatable as string.
            if let Some(f) = replacement.as_object().and_then(|o| o.as_function_object()) {
                let args = [pattern.into(), position.into(), this.into()];
                let v = f.call(activation.global_scope(), &args, activation)?;
                ret.push_str(v.coerce_to_string(activation)?.as_wstr());
            } else {
                let replacement = replacement.coerce_to_string(activation)?;
                ret.push_str(&replacement);
            }
            ret.push_str(&this[position + pattern.len()..]);
            return Ok(AvmString::new(activation.context.gc_context, ret).into());
        } else {
            return Ok(this.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.search`
fn search<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(this), pattern) = (this, args.get(0).unwrap_or(&Value::Undefined)) {
        let this = Value::from(this).coerce_to_string(activation)?;

        let regexp_class = activation.avm2().classes().regexp;
        let pattern = if !pattern.is_of_type(activation, regexp_class) {
            let string = pattern.coerce_to_string(activation)?;
            regexp_class.construct(activation, &[Value::String(string)])?
        } else {
            pattern.coerce_to_object(activation)?
        };

        if let Some(mut regexp) = pattern.as_regexp_mut(activation.context.gc_context) {
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
        };
    }

    Ok(Value::Undefined)
}

/// Implements `String.slice`
fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this = Value::from(this).coerce_to_string(activation)?;
        let start_index = match args.get(0) {
            None => 0,
            Some(n) => {
                let n = n.coerce_to_number(activation)?;
                string_wrapping_index(n, this.len())
            }
        };
        let end_index = match args.get(1) {
            None => this.len(),
            Some(n) => {
                let n = n.coerce_to_number(activation)?;
                string_wrapping_index(n, this.len())
            }
        };
        return if start_index < end_index {
            let ret = WString::from(&this[start_index..end_index]);
            Ok(AvmString::new(activation.context.gc_context, ret).into())
        } else {
            Ok("".into())
        };
    }
    Ok(Value::Undefined)
}

/// Implements `String.split`
fn split<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

        let this = Value::from(this).coerce_to_string(activation)?;
        let limit = match args.get(1).unwrap_or(&Value::Undefined) {
            Value::Undefined => usize::MAX,
            limit => limit.coerce_to_i32(activation)?.max(0) as usize,
        };

        if let Some(mut regexp) = delimiter
            .as_object()
            .as_ref()
            .and_then(|o| o.as_regexp_mut(activation.context.gc_context))
        {
            return Ok(regexp.split(activation, this, limit)?.into());
        }

        let delimiter = delimiter.coerce_to_string(activation)?;

        let storage = if delimiter.is_empty() {
            // When using an empty delimiter, Str::split adds an extra beginning and trailing item, but Flash does not.
            // e.g., split("foo", "") returns ["", "f", "o", "o", ""] in Rust but ["f, "o", "o"] in Flash.
            // Special case this to match Flash's behavior.
            this.iter()
                .take(limit)
                .map(|c| {
                    Value::from(AvmString::new(
                        activation.context.gc_context,
                        WString::from_unit(c),
                    ))
                })
                .collect()
        } else {
            this.split(&delimiter)
                .take(limit)
                .map(|c| Value::from(AvmString::new(activation.context.gc_context, c)))
                .collect()
        };

        return Ok(ArrayObject::from_storage(activation, storage)
            .unwrap()
            .into());
    }
    Ok(Value::Undefined)
}

/// Implements `String.substr`
fn substr<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;

        if args.is_empty() {
            return Ok(Value::from(this));
        }

        let start_index = string_wrapping_index(
            args.get(0)
                .unwrap_or(&Value::Number(0.))
                .coerce_to_number(activation)?,
            this.len(),
        );

        let len = args
            .get(1)
            .unwrap_or(&Value::Number(0x7fffffff as f64))
            .coerce_to_number(activation)?;

        let end_index = if len == f64::INFINITY {
            this.len()
        } else {
            this.len().min(start_index + len as usize)
        };

        let ret = WString::from(&this[start_index..end_index]);
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.substring`
fn substring<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;

        if args.is_empty() {
            return Ok(Value::from(this));
        }

        let mut start_index = string_index(
            args.get(0)
                .unwrap_or(&Value::Number(0.))
                .coerce_to_number(activation)?,
            this.len(),
        );

        let mut end_index = string_index(
            args.get(1)
                .unwrap_or(&Value::Number(0x7fffffff as f64))
                .coerce_to_number(activation)?,
            this.len(),
        );

        if end_index < start_index {
            std::mem::swap(&mut end_index, &mut start_index);
        }

        let ret = WString::from(&this[start_index..end_index]);
        return Ok(AvmString::new(activation.context.gc_context, ret).into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.toLowerCase`
fn to_lower_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;
        return Ok(AvmString::new(
            activation.context.gc_context,
            this.iter()
                .map(crate::string::utils::swf_to_lowercase)
                .collect::<WString>(),
        )
        .into());
    }
    Ok(Value::Undefined)
}

/// Implements `String.toString`
fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        return Ok(Value::from(this));
    }
    Ok(Value::Undefined)
}

/// Implements `String.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        return this.value_of(activation.context.gc_context);
    }
    Ok(Value::Undefined)
}

/// Implements `String.toUpperCase`
fn to_upper_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let this_val = Value::from(this);
        let this = this_val.coerce_to_string(activation)?;
        return Ok(AvmString::new(
            activation.context.gc_context,
            this.iter()
                .map(crate::string::utils::swf_to_uppercase)
                .collect::<WString>(),
        )
        .into());
    }
    Ok(Value::Undefined)
}

/// Construct `String`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace, "String"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
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
    write.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_INSTANCE_PROPERTIES,
    );
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        PUBLIC_INSTANCE_AND_PROTO_METHODS,
    );

    const AS3_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("fromCharCode", from_char_code)];
    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("fromCharCode", from_char_code)];
    write.define_builtin_class_methods(mc, activation.avm2().as3_namespace, AS3_CLASS_METHODS);
    write.define_builtin_class_methods(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_CLASS_METHODS,
    );

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
