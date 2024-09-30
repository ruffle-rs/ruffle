//! `String` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::make_error_1004;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::regexp::{RegExp, RegExpFlags};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::avm2::{ArrayObject, ArrayStorage};
use crate::string::{AvmString, WString};

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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Ok(Value::Undefined)
}

/// Implements `String`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
                None,
                None,
            )
            .into(),
            activation,
        )?;
        proto.set_local_property_is_enumerable(gc_context, (*name).into(), false);
    }
    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get(0)
        .unwrap_or(&Value::String("".into()))
        .coerce_to_string(activation)?
        .into())
}

/// Implements `length` property's getter
fn length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::String(s) = this.value_of(activation.context.gc_context)? {
        return Ok(s.len().into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.charAt`
fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
        let ret = if let Some(c) = s.get(index) {
            activation.strings().make_char(c)
        } else {
            activation.strings().empty()
        };
        return Ok(ret.into());
    }

    Ok(Value::Undefined)
}

/// Implements `String.charCodeAt`
fn char_code_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Ok(Value::Undefined)
}

/// Implements `String.concat`
fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let lhs = Value::from(this).coerce_to_string(activation)?;

    let mut ret = lhs;
    for arg in args {
        let s = arg.coerce_to_string(activation)?;
        ret = AvmString::concat(activation.context.gc_context, ret, s);
    }

    Ok(ret.into())
}

/// Implements `String.fromCharCode`
fn from_char_code<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut out = WString::with_capacity(args.len(), false);
    for arg in args {
        let i = arg.coerce_to_u32(activation)? as u16;
        out.push(i);
    }
    Ok(AvmString::new(activation.context.gc_context, out).into())
}

/// Implements `String.indexOf`
fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let pattern = match args.get(0) {
        None => return Ok(Value::Integer(-1)),
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

/// Implements `String.lastIndexOf`
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

    return this
        .slice(..start_index)
        .unwrap_or(&this)
        .rfind(&pattern)
        .map(|i| Ok(i.into()))
        .unwrap_or_else(|| Ok((-1).into())); // Not found
}

/// Implements String.localeCompare
/// NOTE: Despite the declaration of this function in the documentation, FP does not support multiple strings in comparison
fn locale_compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

/// Implements `String.match`
fn match_s<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let pattern = args.get(0).unwrap_or(&Value::Undefined);
    let this = Value::from(this).coerce_to_string(activation)?;

    let regexp_class = activation.avm2().classes().regexp;
    let pattern = if !pattern.is_of_type(activation, regexp_class.inner_class_definition()) {
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
                    storage.push(AvmString::new(activation.context.gc_context, substring).into());
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
    }

    Ok(Value::Null)
}

/// Implements `String.replace`
fn replace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = Value::from(this).coerce_to_string(activation)?;
    let pattern = args.get(0).unwrap_or(&Value::Undefined);
    let replacement = args.get(1).unwrap_or(&Value::Undefined);

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
            let v = f.call(Value::Null, &args, activation)?;
            ret.push_str(v.coerce_to_string(activation)?.as_wstr());
        } else {
            let replacement = replacement.coerce_to_string(activation)?;
            ret.push_str(&replacement);
        }
        ret.push_str(&this[position + pattern.len()..]);

        Ok(AvmString::new(activation.context.gc_context, ret).into())
    } else {
        Ok(this.into())
    }
}

/// Implements `String.search`
fn search<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let pattern = args.get(0).unwrap_or(&Value::Undefined);
    let this = Value::from(this).coerce_to_string(activation)?;

    let regexp_class = activation.avm2().classes().regexp;
    let pattern = if !pattern.is_of_type(activation, regexp_class.inner_class_definition()) {
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
    }

    Ok(Value::Undefined)
}

/// Implements `String.slice`
fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    if start_index < end_index {
        Ok(activation
            .strings()
            .substring(this, start_index..end_index)
            .into())
    } else {
        Ok("".into())
    }
}

/// Implements `String.split`
fn split<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let delimiter = args.get(0).unwrap_or(&Value::Undefined);

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
            .map(|c| Value::from(activation.strings().make_char(c)))
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

/// Implements `String.substr`
fn substr<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
                return Ok("".into());
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
fn substring<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Ok(activation
        .strings()
        .substring(this, start_index..end_index)
        .into())
}

/// Implements `String.toLowerCase`
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
            .map(crate::string::utils::swf_to_lowercase)
            .collect::<WString>(),
    )
    .into())
}

/// Implements `String.prototype.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_primitive() {
        if let Value::String(v) = *this {
            return Ok(v.into());
        }
    }

    let string_proto = activation.avm2().classes().string.prototype();
    if Object::ptr_eq(string_proto, this) {
        return Ok("".into());
    }

    Err(make_error_1004(activation, "String.prototype.toString"))
}

/// Implements `String.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.value_of(activation.context.gc_context)
}

/// Implements `String.toUpperCase`
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
            .map(crate::string::utils::swf_to_uppercase)
            .collect::<WString>(),
    )
    .into())
}

#[cfg(feature = "test_only_as3")]
fn is_dependent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(prim) = this.as_primitive() {
        if let Value::String(s) = *prim {
            return Ok(s.is_dependent().into());
        }
    }
    panic!();
}

/// Construct `String`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let class = Class::new(
        QName::new(namespaces.public_all(), "String"),
        Some(activation.avm2().class_defs().object),
        Method::from_builtin(instance_init, "<String instance initializer>", mc),
        Method::from_builtin(class_init, "<String class initializer>", mc),
        activation.avm2().class_defs().class,
        mc,
    );

    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);
    class.set_instance_allocator(mc, primitive_allocator);
    class.set_call_handler(
        mc,
        Method::from_builtin(call_handler, "<String call handler>", mc),
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("length", Some(length), None)];
    class.define_builtin_instance_properties(
        mc,
        namespaces.public_all(),
        PUBLIC_INSTANCE_PROPERTIES,
    );
    class.define_builtin_instance_methods(mc, namespaces.as3, PUBLIC_INSTANCE_AND_PROTO_METHODS);

    #[cfg(feature = "test_only_as3")]
    {
        const TEST_METHODS: &[(&str, NativeMethodImpl)] = &[("isDependent", is_dependent)];
        class.define_builtin_instance_methods(mc, namespaces.__ruffle__, TEST_METHODS);
    }

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    class.define_constant_int_class_traits(namespaces.public_all(), CONSTANTS_INT, activation);

    const AS3_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("fromCharCode", from_char_code)];
    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("fromCharCode", from_char_code)];
    class.define_builtin_class_methods(mc, namespaces.as3, AS3_CLASS_METHODS);
    class.define_builtin_class_methods(mc, namespaces.public_all(), PUBLIC_CLASS_METHODS);

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

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
