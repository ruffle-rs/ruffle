//! `JSON` impl

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::globals::array::ArrayIter;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ArrayObject, FunctionObject, Object, TObject};
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
                let mapped_val = match reviver {
                    None => val,
                    Some(reviver) => reviver.call(None, &[key.into(), val], activation)?,
                };
                if matches!(mapped_val, Value::Undefined) {
                    obj.delete_property(activation, &QName::dynamic_name(key).into())?;
                } else {
                    obj.set_property(&QName::dynamic_name(key).into(), mapped_val, activation)?;
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
                    Some(reviver) => reviver.call(None, &[key.into(), val], activation)?,
                };
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
    match reviver {
        None => Ok(val),
        Some(reviver) => reviver.call(None, &["".into(), val], activation),
    }
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
        activation: &mut Activation<'_, 'gc, '_>,
        key: impl Fn() -> AvmString<'gc>,
        value: Value<'gc>,
    ) -> Result<Value<'gc>, Error> {
        let (eval_key, value) = if value.is_primitive() {
            (None, value)
        } else {
            let obj = value.coerce_to_object(activation)?;
            let to_json = obj
                .get_property(
                    &QName::new(Namespace::public(), "toJSON").into(),
                    activation,
                )?
                .coerce_to_object(activation)
                .ok()
                .and_then(|obj| obj.as_function_object());
            if let Some(to_json) = to_json {
                let key = key();
                (Some(key), to_json.call(None, &[key.into()], activation)?)
            } else {
                (None, value)
            }
        };
        if let Some(Replacer::Function(replacer)) = self.replacer {
            replacer.call(
                None,
                &[eval_key.unwrap_or_else(key).into(), value],
                activation,
            )
        } else {
            Ok(value)
        }
    }

    fn serialize_object(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        obj: Object<'gc>,
    ) -> Result<JsonValue, Error> {
        let mut js_obj = JsonObject::new();
        // If the user supplied a PropList, we use that to find properties on the object.
        if let Some(Replacer::PropList(props)) = self.replacer {
            let mut iter = ArrayIter::new(activation, props.into())?;
            while let Some(r) = iter.next(activation) {
                let item = r?.1;
                let key = item.coerce_to_string(activation)?;
                let value =
                    obj.get_property(&QName::new(Namespace::public(), key).into(), activation)?;
                let mapped = self.map_value(activation, || key, value)?;
                if !matches!(mapped, Value::Undefined) {
                    js_obj.insert(
                        &key.to_utf8_lossy(),
                        self.serialize_value(activation, mapped)?,
                    );
                }
            }
        } else {
            for i in 1.. {
                // TODO: We should get more than just enumerable properties
                match obj.get_enumerant_name(i, activation)? {
                    Value::Undefined => break,
                    name_val => {
                        let name = name_val.coerce_to_string(activation)?;
                        let value =
                            obj.get_property(&QName::dynamic_name(name).into(), activation)?;
                        let mapped = self.map_value(activation, || name, value)?;
                        if !matches!(mapped, Value::Undefined) {
                            js_obj.insert(
                                &name.to_utf8_lossy(),
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
        activation: &mut Activation<'_, 'gc, '_>,
        iterable: Object<'gc>,
    ) -> Result<JsonValue, Error> {
        let mut js_arr = Vec::new();
        let mut iter = ArrayIter::new(activation, iterable)?;
        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;
            let mc = activation.context.gc_context;
            let mapped =
                self.map_value(activation, || AvmString::new_utf8(mc, i.to_string()), item)?;
            js_arr.push(self.serialize_value(activation, mapped)?);
        }
        Ok(JsonValue::Array(js_arr))
    }

    fn serialize_value(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
    ) -> Result<JsonValue, Error> {
        Ok(match value {
            Value::Null => JsonValue::Null,
            Value::Undefined => JsonValue::Null,
            Value::Integer(i) => JsonValue::from(i),
            Value::Unsigned(u) => JsonValue::from(u),
            Value::Number(n) => JsonValue::from(n),
            Value::Bool(b) => JsonValue::from(b),
            Value::String(s) => JsonValue::from(s.to_utf8_lossy().deref()),
            Value::Object(obj) => {
                // special case for boxed primitives
                if let Some(prim) = obj.as_primitive() {
                    return self.serialize_value(activation, *prim);
                }
                if self.obj_stack.contains(&obj) {
                    return Err("TypeError: Error #1129: Cyclic structure cannot be converted to JSON string.".into());
                }
                self.obj_stack.push(obj);
                let value = if obj.is_of_type(activation.avm2().classes().array, activation)? {
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
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
    ) -> Result<JsonValue, Error> {
        let mapped = self.map_value(activation, || "".into(), value)?;
        self.serialize_value(activation, mapped)
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
    let replacer = replacer.map(|replacer| {
        if let Some(func) = replacer.as_function_object() {
            Ok(Replacer::Function(func))
        } else if let Some(arr) = replacer.as_array_object() {
            Ok(Replacer::PropList(arr))
        } else {
            Err("TypeError: Error #1131: Replacer argument to JSON stringifier must be an array or a two parameter function.")
        }
    }).transpose()?;

    let mut serializer = AvmSerializer::new(replacer);
    let result = serializer.serialize(activation, *val)?;
    // NOTE: We do not coerce to a string or to a number, the value must already be a string or number.
    let output = if let Value::String(s) = spaces {
        // If the string is empty, just use the normal dump generator.
        if s.is_empty() {
            result.dump()
        } else {
            // we can only use the first 10 characters
            let indent = s.slice(..cmp::min(s.len(), 10)).unwrap().to_utf8_lossy();
            let mut gen = FlashStrGenerator::new(&indent);
            gen.write_json(&result).expect("Can't fail");
            gen.consume()
        }
    } else {
        let indent_size = spaces
            .as_number(activation.context.gc_context)
            .unwrap_or(0.0)
            .clamp(0.0, 10.0) as u16;
        if indent_size == 0 {
            result.dump()
        } else {
            result.pretty(indent_size)
        }
    };
    Ok(AvmString::new_utf8(activation.context.gc_context, output).into())
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
