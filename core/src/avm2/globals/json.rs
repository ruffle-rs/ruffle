//! `JSON` impl

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};
use json::{parse as parse_json, JsonValue};

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
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
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
