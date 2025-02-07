//! `JSON` impl

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::error::{syntax_error, type_error};
use crate::avm2::globals::array::ArrayIter;
use crate::avm2::object::{ArrayObject, FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::{AvmString, Units};
use ruffle_macros::istr;
use serde::Serialize;
use serde_json::{Map as JsonObject, Value as JsonValue};
use std::borrow::Cow;
use std::ops::Deref;

fn deserialize_json_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    json: JsonValue,
    reviver: Option<FunctionObject<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match json {
        JsonValue::Null => Value::Null,
        JsonValue::String(s) => AvmString::new_utf8(activation.gc(), s).into(),
        JsonValue::Bool(b) => b.into(),
        JsonValue::Number(number) => {
            let number = number.as_f64().unwrap();
            if number.fract() == 0.0 {
                f64_to_wrapping_i32(number).into()
            } else {
                number.into()
            }
        }
        JsonValue::Object(js_obj) => {
            let obj = ScriptObject::new_object(activation);
            for entry in js_obj.iter() {
                let key = AvmString::new_utf8(activation.gc(), entry.0);
                let val = deserialize_json_inner(activation, entry.1.clone(), reviver)?;
                let mapped_val = match reviver {
                    None => val,
                    Some(reviver) => reviver.call(activation, Value::Null, &[key.into(), val])?,
                };
                if matches!(mapped_val, Value::Undefined) {
                    obj.delete_string_property_local(key, activation)?;
                } else {
                    obj.set_string_property_local(key, mapped_val, activation)?;
                }
            }
            obj.into()
        }
        JsonValue::Array(js_arr) => {
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(js_arr.len());
            for (key, val) in js_arr.iter().enumerate() {
                let val = deserialize_json_inner(activation, val.clone(), reviver)?;
                let mapped_val = match reviver {
                    None => val,
                    Some(reviver) => reviver.call(activation, Value::Null, &[key.into(), val])?,
                };
                arr.push(Some(mapped_val));
            }
            let storage = ArrayStorage::from_storage(arr);
            let array = ArrayObject::from_storage(activation, storage);
            array.into()
        }
    })
}

fn deserialize_json<'gc>(
    activation: &mut Activation<'_, 'gc>,
    json: JsonValue,
    reviver: Option<FunctionObject<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let val = deserialize_json_inner(activation, json, reviver)?;
    match reviver {
        None => Ok(val),
        Some(reviver) => {
            let args = [istr!("").into(), val];
            reviver.call(activation, Value::Null, &args)
        }
    }
}

enum Replacer<'gc> {
    Function(FunctionObject<'gc>),
    PropList(ArrayObject<'gc>),
}

struct AvmSerializer<'gc> {
    /// This object stack will be used to detect circular references and return an error instead of a panic.
    obj_stack: Vec<Object<'gc>>,
    replacer: Option<Replacer<'gc>>,
}

impl<'gc> AvmSerializer<'gc> {
    fn new(replacer: Option<Replacer<'gc>>) -> Self {
        Self {
            obj_stack: Vec::new(),
            replacer,
        }
    }

    /// Map a value using a toJSON implementation, and then a replacer function.
    ///
    /// First the toJSON method will be called on the value, and the `key` parameter will be passed to it.
    /// If toJSON does not exist, or toJSON is not a function, this step will be skipped.
    ///
    /// The returned value from toJSON (or the original value if that step was skipped) will be passed
    /// to the replacer function with the key in a (key, value) pair, and the value is mapped to the return value
    /// of the replacer function. If the user did not supply a replacer function, this step is skipped.
    ///
    /// The `key` is lazily evaluated because it may be expensive in some areas to generate the key, but the key is
    /// only used if either the `toJSON` step or replacer function step happens, so we only need to evaluate the key there.
    fn map_value(
        &self,
        activation: &mut Activation<'_, 'gc>,
        key: impl Fn() -> AvmString<'gc>,
        value: Value<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let (eval_key, value) = if value.as_object().is_some() {
            if value.has_public_property(istr!("toJSON"), activation) {
                let key = key();
                (
                    Some(key),
                    value.call_public_property(istr!("toJSON"), &[key.into()], activation)?,
                )
            } else {
                (None, value)
            }
        } else {
            (None, value)
        };

        if let Some(Replacer::Function(replacer)) = self.replacer {
            replacer.call(
                activation,
                Value::Null,
                &[eval_key.unwrap_or_else(key).into(), value],
            )
        } else {
            Ok(value)
        }
    }

    fn serialize_object(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        obj: Object<'gc>,
    ) -> Result<JsonValue, Error<'gc>> {
        let mut js_obj = JsonObject::new();
        // If the user supplied a PropList, we use that to find properties on the object.
        if let Some(Replacer::PropList(props)) = self.replacer {
            let mut iter = ArrayIter::new(activation, props.into())?;
            while let Some((_, item)) = iter.next(activation)? {
                let key = item.coerce_to_string(activation)?;
                let value = Value::from(obj).get_public_property(key, activation)?;
                let mapped = self.map_value(activation, || key, value)?;
                if !matches!(mapped, Value::Undefined) {
                    js_obj.insert(
                        key.to_utf8_lossy().into_owned(),
                        self.serialize_value(activation, mapped)?,
                    );
                }
            }
        } else {
            for (name, val) in obj.public_vtable_properties(activation)? {
                let mapped = self.map_value(activation, || name, val)?;
                if !matches!(mapped, Value::Undefined) {
                    js_obj.insert(
                        name.to_utf8_lossy().into_owned(),
                        self.serialize_value(activation, mapped)?,
                    );
                }
            }
            for i in 1.. {
                match obj.get_enumerant_name(i, activation)? {
                    Value::Null => break,
                    name_val => {
                        let name = name_val.coerce_to_string(activation)?;
                        let value = obj.get_enumerant_value(i, activation)?;
                        let mapped = self.map_value(activation, || name, value)?;
                        if !matches!(mapped, Value::Undefined) {
                            js_obj.insert(
                                name.to_utf8_lossy().into_owned(),
                                self.serialize_value(activation, mapped)?,
                            );
                        }
                    }
                }
            }
        }
        Ok(JsonValue::Object(js_obj))
    }

    /// Serializes any object that can be iterated using an ArrayIter (like Array, Vector, etc).
    /// Note that this doesn't actually check if the object passed can be iterated using ArrayIter, it just assumes it can.
    fn serialize_iterable(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        iterable: Object<'gc>,
    ) -> Result<JsonValue, Error<'gc>> {
        let mut js_arr = Vec::new();
        let mut iter = ArrayIter::new(activation, iterable)?;
        while let Some((i, item)) = iter.next(activation)? {
            let mc = activation.gc();
            let mapped =
                self.map_value(activation, || AvmString::new_utf8(mc, i.to_string()), item)?;
            js_arr.push(self.serialize_value(activation, mapped)?);
        }
        Ok(JsonValue::Array(js_arr))
    }

    fn serialize_value(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
    ) -> Result<JsonValue, Error<'gc>> {
        Ok(match value {
            Value::Null => JsonValue::Null,
            Value::Undefined => JsonValue::Null,
            Value::Integer(i) => JsonValue::from(i),
            Value::Number(n) => JsonValue::from(n),
            Value::Bool(b) => JsonValue::from(b),
            Value::String(s) => JsonValue::from(s.to_utf8_lossy().deref()),
            Value::Object(obj) => {
                if self.obj_stack.contains(&obj) {
                    return Err(Error::AvmError(type_error(
                        activation,
                        "Error #1129: Cyclic structure cannot be converted to JSON string.",
                        1129,
                    )?));
                }
                self.obj_stack.push(obj);
                let value = if obj.is_of_type(activation.avm2().class_defs().array) {
                    // TODO: Vectors
                    self.serialize_iterable(activation, obj)?
                } else {
                    self.serialize_object(activation, obj)?
                };
                self.obj_stack
                    .pop()
                    .expect("Stack underflow during JSON serialization");
                value
            }
        })
    }

    /// Same thing as serialize_value, but maps the value before calling it.
    fn serialize(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
    ) -> Result<JsonValue, Error<'gc>> {
        let empty = istr!("");
        let mapped = self.map_value(activation, || empty, value)?;
        self.serialize_value(activation, mapped)
    }
}

/// Implements `JSON.parse`.
pub fn parse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let input = args.get_string(activation, 0)?;
    let reviver = args
        .get_value(1)
        .as_object()
        .map(|o| o.as_function_object().unwrap());

    let parsed = if let Ok(parsed) = serde_json::from_str(&input.to_utf8_lossy()) {
        parsed
    } else {
        return Err(Error::AvmError(syntax_error(
            activation,
            "Error #1132: Invalid JSON parse input.",
            1132,
        )?));
    };

    deserialize_json(activation, parsed, reviver)
}

/// Implements `JSON.stringify`.
pub fn stringify<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let val = args.get_value(0);
    let replacer = args.get_value(1).as_object();
    let spaces = args.get_value(2);

    // If the replacer is None, that means it was either undefined or null.
    if replacer.is_none() && !matches!(args.get_value(1), Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #1131: Replacer argument to JSON stringifier must be an array or a two parameter function.",
            1131,
        )?));
    }

    let replacer = replacer.map(|replacer| {
        if let Some(func) = replacer.as_function_object() {
            Ok(Replacer::Function(func))
        } else if let Some(arr) = replacer.as_array_object() {
            Ok(Replacer::PropList(arr))
        } else {
            Err(Error::AvmError(type_error(
                activation,
                "Error #1131: Replacer argument to JSON stringifier must be an array or a two parameter function.",
                1131,
            )?))
        }
    }).transpose()?;

    // NOTE: We do not coerce to a string or to a number, the value must already be a string or number.
    let indent = if let Value::String(s) = &spaces {
        if s.is_empty() {
            None
        } else {
            // We can only use the first 10 characters.
            let indent = &s[..s.len().min(10)];
            let indent_bytes = match indent.units() {
                Units::Bytes(units) => Cow::Borrowed(units),
                Units::Wide(_) => Cow::Owned(indent.to_utf8_lossy().into_owned().into_bytes()),
            };
            Some(indent_bytes)
        }
    } else if spaces.is_number() {
        let indent_size = spaces.as_f64().clamp(0.0, 10.0) as u16;
        if indent_size == 0 {
            None
        } else {
            Some(Cow::Owned(b" ".repeat(indent_size.into())))
        }
    } else {
        None
    };

    let mut serializer = AvmSerializer::new(replacer);
    let json = serializer.serialize(activation, val)?;
    let result = match indent {
        Some(indent) => {
            let mut result = Vec::with_capacity(128);
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
            let mut serializer = serde_json::Serializer::with_formatter(&mut result, formatter);
            json.serialize(&mut serializer)
                .expect("JSON serialization cannot fail");
            result
        }
        None => serde_json::to_vec(&json).expect("JSON serialization cannot fail"),
    };
    Ok(AvmString::new_utf8_bytes(activation.gc(), &result).into())
}
