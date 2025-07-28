//! `RegExp` impl

use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::type_error;
use crate::avm2::object::{ArrayObject, Object, TObject as _};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::regexp::RegExpFlags;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;

pub use crate::avm2::object::reg_exp_allocator;

/// Implements `RegExp`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut regexp) = this.as_regexp_mut(activation.gc()) {
        let source: AvmString<'gc> = match args.get_value(0) {
            Value::Undefined => istr!(""),
            Value::Object(Object::RegExpObject(o)) => {
                if !matches!(args.get_value(1), Value::Undefined) {
                    return Err(Error::avm_error(type_error(
                        activation,
                        "Error #1100: Cannot supply flags when constructing one RegExp from another.",
                        1100,
                    )?));
                }
                let other = o.regexp();
                regexp.set_source(other.source());
                regexp.set_flags(other.flags());
                return Ok(Value::Undefined);
            }
            arg => arg.coerce_to_string(activation)?,
        };

        regexp.set_source(source);

        let flag_chars = match args.get_value(1) {
            Value::Undefined => istr!(""),
            arg => arg.coerce_to_string(activation)?,
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
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_class = activation.avm2().classes().regexp;

    if let Some(arg) = args.get_optional(0).filter(|_| args.len() == 1) {
        if arg.as_object().and_then(|o| o.as_regexp_object()).is_some() {
            return Ok(arg);
        }
    }
    this_class.construct(activation, args)
}

/// Implements `RegExp.dotall`
pub fn get_dotall<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::DOTALL).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.extended`
pub fn get_extended<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::EXTENDED).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.global`
pub fn get_global<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::GLOBAL).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.ignoreCase`
pub fn get_ignore_case<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::IGNORE_CASE).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.multiline`
pub fn get_multiline<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(regexp) = this.as_regexp() {
        return Ok(regexp.flags().contains(RegExpFlags::MULTILINE).into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.lastIndex`'s getter
pub fn get_last_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(re) = this.as_regexp() {
        return Ok(re.last_index().into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.lastIndex`'s setter
pub fn set_last_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut re) = this.as_regexp_mut(activation.gc()) {
        // FIXME what is the behavior for negative lastIndex?
        let i = args.get_i32(0);
        re.set_last_index(i as usize);
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.source`
pub fn get_source<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(re) = this.as_regexp() {
        return Ok(re.source().into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.exec`
pub fn exec<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut re) = this.as_regexp_mut(activation.gc()) {
        let text = args.get_string(activation, 0);

        let matched = match re.exec(text) {
            Some(matched) => matched,
            None => return Ok(Value::Null),
        };

        let storage = matched
            .groups()
            .map(|range| {
                range.map_or(Value::Undefined, |range| {
                    activation.strings().substring(text, range).into()
                })
            })
            .collect();

        let object = ArrayObject::from_storage(activation, storage);

        for (name, range) in matched.named_groups() {
            let string = range.map_or(istr!(""), |range| {
                activation.strings().substring(text, range)
            });

            object.set_dynamic_property(
                AvmString::new_utf8(activation.gc(), name),
                string.into(),
                activation.gc(),
            );
        }

        object.set_dynamic_property(
            istr!("index"),
            Value::Number(matched.start() as f64),
            activation.gc(),
        );

        object.set_dynamic_property(istr!("input"), text.into(), activation.gc());

        return Ok(object.into());
    }

    Ok(Value::Undefined)
}

/// Implements `RegExp.test`
pub fn test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut re) = this.as_regexp_mut(activation.gc()) {
        let text = args.get_string(activation, 0);
        return Ok(re.test(text).into());
    }

    Ok(Value::Undefined)
}
