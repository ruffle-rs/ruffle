//! `JSON` impl

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::globals::array::{resolve_array_hole, ArrayIter};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ArrayObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};
use json::{
    codegen::Generator as JsonGenerator, object::Object as JsonObject, parse as parse_json,
    JsonValue,
};
use std::cmp;
use std::ops::Deref;

fn deserialize_json_inner<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    json: JsonValue,
    reviver: Option<Object<'gc>>,
) -> Result<Value<'gc>, Error> {
    Ok(match json {
        JsonValue::Null => Value::Null,
        JsonValue::Short(s) => {
            AvmString::new_utf8(activation.context.gc_context, s.to_string()).into()
        }
        JsonValue::String(s) => AvmString::new_utf8(activation.context.gc_context, s).into(),
        JsonValue::Boolean(b) => b.into(),
        JsonValue::Number(num) => {
            let num: f64 = num.into();
            if num.fract() == 0.0 {
                f64_to_wrapping_i32(num).into()
            } else {
                num.into()
            }
        }
        JsonValue::Object(js_obj) => {
            let obj_class = activation.avm2().classes().object;
            let mut obj = obj_class.construct(activation, &[])?;
            for entry in js_obj.iter() {
                let key = AvmString::new_utf8(activation.context.gc_context, entry.0);
                let val = deserialize_json_inner(activation, entry.1.clone(), reviver)?;
                let mapped_val = reviver.map_or(Ok(val.clone()), |reviver| {
                    reviver.call(None, &[key.into(), val], activation)
                })?;
                if matches!(mapped_val, Value::Undefined) {
                    obj.delete_property(activation, &QName::new(Namespace::public(), key).into())?;
                } else {
                    obj.set_property(
                        obj,
                        &QName::new(Namespace::public(), key).into(),
                        mapped_val,
                        activation,
                    )?;
                }
            }
            obj.into()
        }
        JsonValue::Array(js_arr) => {
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(js_arr.len());
            for (key, val) in js_arr.iter().enumerate() {
                let val = deserialize_json_inner(activation, val.clone(), reviver)?;
                let mapped_val = reviver.map_or(Ok(val.clone()), |reviver| {
                    reviver.call(None, &[key.into(), val], activation)
                })?;
                arr.push(Some(mapped_val));
            }
            let storage = ArrayStorage::from_storage(arr);
            let array = ArrayObject::from_storage(activation, storage)?;
            array.into()
        }
    })
}

fn deserialize_json<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    json: JsonValue,
    reviver: Option<Object<'gc>>,
) -> Result<Value<'gc>, Error> {
    let val = deserialize_json_inner(activation, json, reviver)?;
    reviver.map_or(Ok(val.clone()), |reviver| {
        reviver.call(None, &["".into(), val], activation)
    })
}

/// This is a custom generator backend for the json crate that allows modifying the indentation character.
/// This generator is used in `JSON.stringify` when a string is passed to the `space` parameter.
struct FlashStrGenerator<'a> {
    code: Vec<u8>,
    indent_str: &'a str,
    indent_size: u16,
}

impl<'a> FlashStrGenerator<'a> {
    fn new(indent_str: &'a str) -> Self {
        Self {
            code: Vec::with_capacity(1024),
            indent_str,
            indent_size: 0,
        }
    }

    pub fn consume(self) -> String {
        // SAFETY: JSON crate should never generate invalid UTF-8
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl<'a> JsonGenerator for FlashStrGenerator<'a> {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> std::io::Result<()> {
        self.code.extend_from_slice(slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> std::io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> std::io::Result<()> {
        self.code.extend_from_slice(slice);
        Ok(())
    }

    fn new_line(&mut self) -> std::io::Result<()> {
        self.code.push(b'\n');
        for _ in 0..self.indent_size {
            self.code.extend_from_slice(self.indent_str.as_bytes());
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.indent_size += 1;
    }

    fn dedent(&mut self) {
        self.indent_size -= 1;
    }
}

struct AvmSerializer {
    /// This object stack will be used to detect circular references and return an error instead of a panic.
    obj_stack: Vec<*const ObjectPtr>,
}

impl AvmSerializer {
    fn new() -> Self {
        Self {
            obj_stack: Vec::new(),
        }
    }

    /// Maps a value using a replacer object. A replacer object can be either an Array or a Function.
    ///
    /// If the replacer object is an Array, we return `value` if `name` appears in the array, otherwise we return None.
    /// The value in the array must be either a string or a number (including int, uint, etc). All other types will be skipped.
    /// If the replacer object is an Array and `allow_array` is false, the replacer object is ignored and `value` is immediately returned.
    ///
    /// If the replacer object is a Function, we pass the `name` and `value` parameters to the function, and
    /// the returned value will be used instead of the original value.
    ///
    /// If the replacer object is neither a Function or Array, an error is returned.
    fn map_value<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        value: Value<'gc>,
        replacer: Object<'gc>,
        allow_array: bool,
    ) -> Result<Option<Value<'gc>>, Error> {
        if let Some(arr) = replacer.as_array_storage() {
            if allow_array {
                for (i, item) in arr.iter().enumerate() {
                    let item = resolve_array_hole(activation, replacer, i, item)?;
                    let val = match item {
                        Value::String(s) => s,
                        // NOTE: coerce_to_string cannot execute user code because item is a number
                        _ if item.is_number() => item.coerce_to_string(activation)?,
                        _ => continue,
                    };
                    if val == name {
                        return Ok(Some(value));
                    }
                }
            } else {
                return Ok(Some(value));
            }
            Ok(None)
        } else if let Some(func) = replacer.as_executable() {
            let mapped = func.exec(None, &[name.into(), value], activation, None, replacer)?;
            Ok(Some(mapped))
        } else {
            Err("TypeError: Error #1131: Replacer argument to JSON stringifier must be an array or a two parameter function.".into())
        }
    }

    fn serialize_json_inner<'gc>(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
        replacer: Option<Object<'gc>>,
    ) -> Result<JsonValue, Error> {
        Ok(match value {
            Value::Null => JsonValue::Null,
            Value::Undefined => JsonValue::Null,
            Value::Integer(i) => JsonValue::from(i),
            Value::Unsigned(u) => JsonValue::from(u),
            Value::Number(n) => JsonValue::from(n),
            Value::Bool(b) => JsonValue::from(b),
            Value::String(s) => JsonValue::from(s.deref()),
            Value::Object(obj) => {
                // special case for boxed primitives
                if let Some(prim) = obj.as_primitive() {
                    return self.serialize_json_inner(activation, prim.deref().clone(), replacer);
                }
                if self.obj_stack.contains(&obj.as_ptr()) {
                    return Err("TypeError: Error #1129: Cyclic structure cannot be converted to JSON string.".into());
                }
                self.obj_stack.push(obj.as_ptr());
                let value = if obj.is_of_type(activation.avm2().classes().array, activation)? {
                    let mut arr = Vec::new();
                    let mut iter = ArrayIter::new(activation, obj)?;
                    while let Some(r) = iter.next(activation) {
                        let (i, item) = r?;
                        if let Some(mapped) =
                            replacer.map_or(Ok(Some(item.clone())), |replacer| {
                                self.map_value(
                                    activation,
                                    AvmString::new(activation.context.gc_context, i.to_string()),
                                    item,
                                    replacer,
                                    false,
                                )
                            })?
                        {
                            arr.push(self.serialize_json_inner(activation, mapped, replacer)?);
                        }
                    }
                    JsonValue::Array(arr)
                } else {
                    let prop = obj
                        .get_property(obj, &QName::new(Namespace::public(), "toJSON"), activation)?
                        .coerce_to_object(activation)
                        .ok();
                    if let Some(to_json) = prop.and_then(|obj| obj.as_executable()) {
                        // If the object contains a `toJSON` property, and it is executable,
                        // we execute that and serialize the returned value.
                        let val = to_json.exec(None, &[], activation, None, prop.unwrap())?;
                        self.serialize_json_inner(activation, val, replacer)?
                    } else {
                        // If this object does not have a `toJSON` property, or `toJSON` isn't executable, we iterate
                        // the enumerable properties of the object and collect the output into a JsonObject.
                        let mut js_obj = JsonObject::new();
                        for i in 1.. {
                            if let Some(name) = obj.get_enumerant_name(i) {
                                let value = obj.get_property(obj, &name, activation)?;
                                // NOTE: This is the only area where `allow_array` is enabled in `self.map_value`.
                                if let Some(mapped) =
                                    replacer.map_or(Ok(Some(value.clone())), |replacer| {
                                        self.map_value(
                                            activation,
                                            name.local_name(),
                                            value,
                                            replacer,
                                            true,
                                        )
                                    })?
                                {
                                    let js_val =
                                        self.serialize_json_inner(activation, mapped, replacer)?;
                                    js_obj.insert(&name.local_name(), js_val);
                                };
                            } else {
                                break;
                            }
                        }
                        JsonValue::Object(js_obj)
                    }
                };
                self.obj_stack
                    .pop()
                    .expect("Stack underflow during JSON serialization");
                value
            }
        })
    }

    /// Serializes an AVM value to a JsonValue. The replacer object will be used for
    /// customizing the output (see map_value doc comments for more details).
    /// Data structures that use circular references will be detected and an error will be returned.
    fn serialize_json<'gc>(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
        replacer: Option<Object<'gc>>,
    ) -> Result<JsonValue, Error> {
        if let Some(mapped) = replacer.map_or(Ok(Some(value.clone())), |replacer| {
            self.map_value(activation, "".into(), value, replacer, false)
        })? {
            self.serialize_json_inner(activation, mapped, replacer)
        } else {
            Ok(JsonValue::Null)
        }
    }
}

/// Implements `JSON`'s instance initializer.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("ArgumentError: Error #2012: JSON class cannot be instantiated.".into())
}

/// Implements `JSON`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `JSON.parse`.
pub fn parse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let input = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let reviver = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)
        .ok();
    let parsed = parse_json(&input.to_utf8_lossy())
        .map_err(|_| "SyntaxError: Error #1132: Invalid JSON parse input.")?;
    deserialize_json(activation, parsed, reviver)
}

/// Implements `JSON.stringify`.
pub fn stringify<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let val = args.get(0).unwrap_or(&Value::Undefined);
    let replacer = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)
        .ok();
    let spaces = args.get(2).unwrap_or(&Value::Undefined);
    let mut serializer = AvmSerializer::new();
    let result = serializer.serialize_json(activation, val.clone(), replacer)?;
    // NOTE: We do not coerce to a string or to a number, the value must already be a string or number.
    let output = if let Value::String(s) = spaces {
        // If the string is empty, just use the normal dump generator.
        if s.is_empty() {
            result.dump()
        } else {
            // we can only use the first 10 characters
            let indent = s.get(..cmp::min(s.len(), 10)).unwrap();
            let mut gen = FlashStrGenerator::new(indent);
            gen.write_json(&result).expect("Can't fail");
            gen.consume()
        }
    } else {
        let indent_size = spaces
            .as_number(activation.context.gc_context)
            .unwrap_or(0.0);
        let indent_size = if indent_size.is_sign_negative() {
            0.0
        } else {
            indent_size
        } as u16;
        if indent_size == 0 {
            result.dump()
        } else {
            result.pretty(indent_size)
        }
    };
    Ok(AvmString::new(activation.context.gc_context, output).into())
}

/// Construct `JSON`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "JSON"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<JSON instance initializer>", mc),
        Method::from_builtin(class_init, "<JSON class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] =
        &[("parse", parse), ("stringify", stringify)];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);
    class
}
