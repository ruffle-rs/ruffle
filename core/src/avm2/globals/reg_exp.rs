//! `RegExp` impl

use crate::avm2::error::type_error;
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::regexp::RegExpFlags;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::{activation::Activation, array::ArrayStorage};
use crate::string::{AvmString, WString};

pub use crate::avm2::object::reg_exp_allocator;

/// Implements `RegExp`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut regexp) = this.as_regexp_mut(activation.context.gc_context) {
        let source: AvmString<'gc> = match args.get(0) {
            Some(Value::Undefined) => activation.strings().empty(),
            Some(Value::Object(Object::RegExpObject(o))) => {
                if !matches!(args.get(1), Some(Value::Undefined)) {
                    return Err(Error::AvmError(type_error(
                        activation,
                        "Error #1100: Cannot supply flags when constructing one RegExp from another.",
                        1100,
                    )?));
                }
                let other = o.as_regexp().unwrap();
                regexp.set_source(other.source());
                regexp.set_flags(other.flags());
                return Ok(Value::Undefined);
            }
            Some(arg) => arg.coerce_to_string(activation)?,
            None => activation.strings().empty(),
        };

        regexp.set_source(source);

        let flag_chars = match args.get(1) {
            None | Some(Value::Undefined) => activation.strings().empty(),
            Some(arg) => arg.coerce_to_string(activation)?,
        };

        let mut flags = RegExpFlags::empty();
        for c in &flag_chars {
            flags |= match u8::try_from(c) {
                Ok(b's') => RegExpFlags::DOTALL,
                Ok(b'x') => RegExpFlags::EXTENDED,
                Ok(b'g') => RegExpFlags::GLOBAL,
                Ok(b'i') => RegExpFlags::IGNORE_CASE,
                Ok(b'm') => RegExpFlags::MULTILINE,
                _ => continue,
            };
        }

        regexp.set_flags(flags);
    }

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_class = activation.avm2().classes().regexp;

    if args.len() == 1 {
        let arg = args.get(0).cloned().unwrap();
        if arg.as_object().and_then(|o| o.as_regexp_object()).is_some() {
            return Ok(arg);
        }
    }
    this_class.construct(activation, args).map(|o| o.into())
}

/// Implements `RegExp.dotall`
pub fn get_dotall<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::DOTALL).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.extended`
pub fn get_extended<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::EXTENDED).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.global`
pub fn get_global<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::GLOBAL).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.ignoreCase`
pub fn get_ignore_case<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::IGNORE_CASE).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.multiline`
pub fn get_multiline<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::MULTILINE).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.lastIndex`'s getter
pub fn get_last_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(re) = this.as_regexp() {
        return Ok(re.last_index().into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.lastIndex`'s setter
pub fn set_last_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut re) = this.as_regexp_mut(activation.context.gc_context) {
        let i = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        re.set_last_index(i as usize);
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.source`
pub fn get_source<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(re) = this.as_regexp() {
        return Ok(re.source().into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.exec`
pub fn exec<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut re) = this.as_regexp_mut(activation.context.gc_context) {
        let text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        let (storage, index) = match re.exec(text) {
            Some(matched) => {
                let substrings = matched
                    .groups()
                    .map(|range| range.map(|r| WString::from(&text[r])));

                let storage = ArrayStorage::from_iter(substrings.map(|s| match s {
                    None => Value::Undefined,
                    Some(s) => AvmString::new(activation.context.gc_context, s).into(),
                }));

                (storage, matched.start())
            }
            None => return Ok(Value::Null),
        };

        let object = ArrayObject::from_storage(activation, storage)?;

        object.set_string_property_local("index", Value::Number(index as f64), activation)?;

        object.set_string_property_local("input", text.into(), activation)?;

        return Ok(object.into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.test`
pub fn test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut re) = this.as_regexp_mut(activation.context.gc_context) {
        let text = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;
        return Ok(re.test(text).into());
    }

    Ok(Value::Undefined)
}
