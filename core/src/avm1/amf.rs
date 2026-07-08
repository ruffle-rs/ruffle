//! AVM1 AMF serialization and deserialization code.
//!
//! Used in objects such as NetConnection, LocalConnection, or SharedObject
use crate::avm1::{Activation, Attribute, NativeObject, Object, Value};
use crate::string::AvmString;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf0::writer::{Amf0Writer, CacheKey, ObjWriter};
use flash_lso::types::{ClassDefinition, Element, ObjectId, Reference, Value as AmfValue};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AmfConfig {
    pub supports_strict_arrays: bool,
}

/// Analyze array keys to determine max length and presence of custom properties
fn analyze_array_keys<'gc>(keys: &[AvmString<'gc>], initial_len: usize) -> (usize, bool) {
    let mut length = initial_len;
    let mut has_custom = false;

    for key in keys {
        match key.to_utf8_lossy().parse::<usize>() {
            Ok(index) => length = length.max(index + 1),
            Err(_) => has_custom = true,
        }
    }
    (length, has_custom)
}

/// Get class alias of object
fn object_class_alias<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Option<String> {
    // Check for a mutated local 'constructor' property (matches Flash's forged_ctor behavior)
    // Fall back to the hidden '__constructor__' property to bypass the prototype chain
    // (matches Flash's forged_proto behavior by ignoring dynamic __proto__ reassignments)
    let constructor_key = AvmString::new_utf8(activation.gc(), "constructor");
    let magic_constructor_key = AvmString::new_utf8(activation.gc(), "__constructor__");

    let ctor = if object.has_own_property(activation, constructor_key) {
        object.get(constructor_key, activation)
    } else {
        object.get(magic_constructor_key, activation)
    };

    match ctor {
        Ok(Value::Object(ctor_obj)) => activation
            .context
            .avm1
            .get_alias_by_constructor(activation.swf_version(), ctor_obj)
            .map(|alias| alias.to_utf8_lossy().into_owned()),
        _ => None,
    }
}

/// Serialize AMF data in NetConnection.addHeader, NetConnection.call, and LocalConnection.send
pub fn serialize<'gc>(activation: &mut Activation<'_, 'gc>, value: Value<'gc>) -> AmfValue {
    match value {
        Value::Undefined => AmfValue::Undefined,
        Value::Null => AmfValue::Null,
        Value::Bool(bool) => AmfValue::Bool(bool),
        Value::Number(number) => AmfValue::Number(number),
        Value::String(string) => AmfValue::String(string.to_string()),
        Value::Object(object) => {
            if object.as_display_object().is_some() {
                AmfValue::Undefined
            } else if let NativeObject::Array(_) = object.native() {
                serialize_array(activation, object)
            } else if let Some(xml_node) = object.as_xml_node() {
                let string = xml_node
                    .into_string(activation)
                    .expect("Failed to convert xml to string in SharedObject");
                AmfValue::XML(string.to_utf8_lossy().into_owned(), true)
            } else if let NativeObject::Date(date) = object.native() {
                AmfValue::Date(date.get().time(), None)
            } else {
                let class_def =
                    object_class_alias(activation, object).map(ClassDefinition::default_with_name);
                let elements = serialize_object_properties(activation, object);
                AmfValue::Object(ObjectId::INVALID, elements, class_def)
            }
        }
        Value::MovieClip(_) => AmfValue::Undefined,
    }
}

/// Serialize an Object in NetConnection or LocalConnection
fn serialize_object_properties<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Vec<Element> {
    let mut w = Amf0Writer::default();
    let config = AmfConfig {
        supports_strict_arrays: true,
    };
    recursive_serialize(activation, object, &mut w, config);
    w.commit_lso("root").into_iter().collect()
}

/// Serialize an array for NetConnection and LocalConnection
fn serialize_array<'gc>(activation: &mut Activation<'_, 'gc>, array: Object<'gc>) -> AmfValue {
    let initial_len = array.length(activation).unwrap_or(0).max(0) as usize;
    let keys = array.get_keys(activation, false);
    let (length, has_custom_properties) = analyze_array_keys(&keys, initial_len);

    if has_custom_properties {
        // Mixed Array: Has true non-numeric keys
        let associative = keys
            .into_iter()
            .rev()
            .map(|key| {
                let prop_value = array.get(key, activation).unwrap_or(Value::Undefined);
                Element::new(key.to_string(), Rc::new(serialize(activation, prop_value)))
            })
            .collect();

        AmfValue::ECMAArray(ObjectId::INVALID, Vec::new(), associative, length as u32)
    } else {
        // Pure Dense Array: Pad holes with Undefined to maintain contiguous indices
        let dense = (0..length)
            .map(|i| {
                let elem_name = AvmString::new_utf8(activation.gc(), i.to_string());
                let prop_value = array.get(elem_name, activation).unwrap_or(Value::Undefined);
                Rc::new(serialize(activation, prop_value))
            })
            .collect();

        AmfValue::StrictArray(ObjectId::INVALID, dense)
    }
}

/// Helper to serialize a specific value into the ObjWriter cache architecture.
fn serialize_value_to_writer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &str,
    elem: Value<'gc>,
    writer: &mut dyn ObjWriter<'_>,
    config: AmfConfig,
) {
    match elem {
        Value::Object(o) => {
            if o.as_function().is_some() {
                // Flash entirely skips functions during object serialization
            } else if o.as_display_object().is_some() {
                writer.undefined(name)
            } else if let NativeObject::Array(_) = o.native() {
                let keys = o.get_keys(activation, false);
                let initial_len = o.length(activation).unwrap_or(0).max(0) as usize;
                let (length, has_custom_properties) = analyze_array_keys(&keys, initial_len);

                if has_custom_properties || !config.supports_strict_arrays {
                    let (aw, token) = writer.array(CacheKey::from_ptr(o.as_ptr()));
                    if let Some(mut aw) = aw {
                        recursive_serialize(activation, o, &mut aw, config);
                        aw.commit(name, length as u32);
                    } else {
                        writer.reference(name, token);
                    }
                } else {
                    let (aw, token) = writer.strict_array(CacheKey::from_ptr(o.as_ptr()));
                    if let Some(mut aw) = aw {
                        for i in 0..length {
                            let elem_name = AvmString::new_utf8(activation.gc(), i.to_string());
                            let prop_value =
                                o.get(elem_name, activation).unwrap_or(Value::Undefined);
                            serialize_value_to_writer(
                                activation,
                                &i.to_string(),
                                prop_value,
                                &mut aw,
                                config,
                            );
                        }
                        aw.commit(name);
                    } else {
                        writer.reference(name, token);
                    }
                }
            } else if let Some(xml_node) = o.as_xml_node() {
                // TODO: What happens if an exception is thrown here?
                let string = xml_node
                    .into_string(activation)
                    .expect("Failed to convert xml to string in SharedObject");
                writer.xml(name, string.to_utf8_lossy().as_ref(), true)
            } else if let NativeObject::Date(date) = o.native() {
                writer.date(name, date.get().time(), None)
            } else {
                let class_alias = object_class_alias(activation, o);
                let key = CacheKey::from_ptr(o.as_ptr());

                if let Some(alias) = class_alias {
                    let (ow, token) = writer.typed_object(&alias, key);
                    if let Some(mut ow) = ow {
                        recursive_serialize(activation, o, &mut ow, config);
                        ow.commit(name);
                    } else {
                        writer.reference(name, token);
                    }
                } else {
                    let (ow, token) = writer.object(key);
                    if let Some(mut ow) = ow {
                        recursive_serialize(activation, o, &mut ow, config);
                        ow.commit(name);
                    } else {
                        writer.reference(name, token);
                    }
                }
            }
        }
        Value::Number(f) => writer.number(name, f),
        Value::String(s) => writer.string(name, s.to_utf8_lossy().as_ref()),
        Value::Undefined | Value::MovieClip(_) => writer.undefined(name),
        Value::Null => writer.null(name),
        Value::Bool(b) => writer.bool(name, b),
    }
}

/// Serialize an Object and any children to a JSON object
/// Used to serialize objects for NetConnection and LocalConnection Flash Remoting
/// As well as for SharedObject LSO
pub fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    writer: &mut dyn ObjWriter<'_>,
    config: AmfConfig,
) {
    // Reversed to match flash player ordering
    for element_name in obj.get_keys(activation, false).into_iter().rev() {
        if let Ok(elem) = obj.get(element_name, activation) {
            let name = element_name.to_utf8_lossy();
            serialize_value_to_writer(activation, name.as_ref(), elem, writer, config);
        }
    }
}

/// Deserialize a AmfValue to a Value
pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    lso: &AMF0Decoder,
    reference_cache: &mut BTreeMap<Reference, Value<'gc>>,
) -> Value<'gc> {
    match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.gc(), s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ECMAArray(_, _, associative, len) => {
            let array_constructor = activation.prototypes().array_constructor;
            if let Ok(Value::Object(obj)) =
                array_constructor.construct(activation, &[(*len).into()])
            {
                let v: Value<'gc> = obj.into();

                // This should always be valid, but lets be sure
                if let Some(reference) = lso.as_reference(val) {
                    reference_cache.insert(reference, v);
                }

                for entry in associative {
                    let value = deserialize_value(activation, entry.value(), lso, reference_cache);

                    if let Ok(i) = entry.name().parse::<i32>() {
                        obj.set_element(activation, i, value).unwrap();
                    } else {
                        obj.define_value(
                            activation.gc(),
                            AvmString::new_utf8(activation.gc(), &entry.name),
                            value,
                            Attribute::empty(),
                        );
                    }
                }

                v
            } else {
                Value::Undefined
            }
        }
        AmfValue::StrictArray(_, values) => {
            let array_constructor = activation.prototypes().array_constructor;
            let obj = array_constructor
                .construct(activation, &[Value::from_usize_lossy(values.len())])
                .expect("AVM1 Array constructor should be infallible")
                .as_object(activation)
                .expect("AVM1 Array constructor should return an object");

            let v: Value<'gc> = obj.into();
            if let Some(reference) = lso.as_reference(val) {
                reference_cache.insert(reference, v);
            }

            // Only define properties for elements that are NOT AMF Undefined holes
            for (i, item) in values.iter().enumerate() {
                let value = deserialize_value(activation, item, lso, reference_cache);
                if !matches!(value, Value::Undefined) {
                    obj.set_element(activation, i as i32, value).unwrap();
                }
            }

            v
        }

        AmfValue::Object(_, elements, _) => {
            // Deserialize Object
            let obj = Object::new(
                &activation.context.strings,
                Some(activation.prototypes().object),
            );

            let v: Value<'gc> = obj.into();

            // This should always be valid, but lets be sure
            if let Some(reference) = lso.as_reference(val) {
                reference_cache.insert(reference, v);
            }

            for entry in elements {
                let value = deserialize_value(activation, entry.value(), lso, reference_cache);
                let name = AvmString::new_utf8(activation.gc(), &entry.name);
                obj.define_value(activation.gc(), name, value, Attribute::empty());
            }

            v
        }
        AmfValue::Date(time, _) => {
            let date_proto = activation.prototypes().date_constructor;

            if let Ok(Value::Object(obj)) = date_proto.construct(activation, &[(*time).into()]) {
                Value::Object(obj)
            } else {
                Value::Undefined
            }
        }
        AmfValue::XML(content, _) => {
            let xml_proto = activation.prototypes().xml_constructor;

            if let Ok(Value::Object(obj)) = xml_proto.construct(
                activation,
                &[Value::String(AvmString::new_utf8(activation.gc(), content))],
            ) {
                Value::Object(obj)
            } else {
                Value::Undefined
            }
        }
        AmfValue::Reference(x) => {
            // This should always be a valid reference, but a "bad" file could create an invalid one
            // In that case we will just assume undefined
            let val = reference_cache.get(x).unwrap_or(&Value::Undefined);
            *val
        }
        _ => Value::Undefined,
    }
}
