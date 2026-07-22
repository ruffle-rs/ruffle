use crate::avm1::{Activation, Attribute, NativeObject, Object, Value};
use crate::string::AvmString;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf0::writer::{Amf0Writer, CacheKey, ObjWriter};
use flash_lso::types::{ClassDefinition, Element, ObjectId, Reference, Value as AmfValue};
use ruffle_macros::istr;
use std::collections::BTreeMap;
use std::rc::Rc;

/// AMF0 serializes an object as a TypedObject if its constructor has a class
/// name registered through `Object.registerClass`; otherwise it serializes it as
/// an anonymous object. The registered name is written as the AMF0 class name.
/// Flash determines this name by resolving the object's constructor, avoiding
/// prototype-chain modifications and accessor invocation. This mirrors that
/// behavior by locating the constructor and looking up its registered class name.
fn object_class_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Option<String> {
    let constructor_key = istr!("constructor");
    let magic_constructor_key = istr!("__constructor__");
    // 1. Check for a mutated local 'constructor' property first.
    // 2. Fall back to the hidden '__constructor__' property (which may be inherited).
    // ASSetPropFlags attributes have no effect on typed object resolution.
    // Instead, as done here, we must look up "constructor"/"__constructor__ directly.
    let ctor = if object.has_own_property(activation, constructor_key) {
        if object.has_own_virtual(activation, constructor_key) {
            None
        } else {
            object.get(constructor_key, activation).ok()
        }
    } else {
        object.get_stored_property(activation, magic_constructor_key)
    };

    match ctor {
        Some(Value::Object(ctor_obj)) => activation
            .context
            .avm1
            .get_class_name_by_constructor(activation.swf_version(), ctor_obj)
            .map(|class_name| class_name.to_utf8_lossy().into_owned()),
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
                    object_class_name(activation, object).map(ClassDefinition::default_with_name);
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
    recursive_serialize(activation, object, &mut w);
    w.commit_lso("root").into_iter().collect()
}

/// Serialize an array for NetConnection and LocalConnection
fn serialize_array<'gc>(activation: &mut Activation<'_, 'gc>, array: Object<'gc>) -> AmfValue {
    let mut length = array.length(activation).unwrap_or(0).max(0) as usize;
    let keys = array.get_keys(activation, false);

    // Flash treats any numeric key as an array element. If a numeric key
    // exceeds the current length, it expands the array's serialization length.
    for key in &keys {
        if let Ok(index) = key.to_utf8_lossy().parse::<usize>() {
            length = length.max(index + 1);
        }
    }

    let mut has_custom_properties = false;
    let mut associative = Vec::new();

    // Flash expects insertion order for these properties
    for key in keys.into_iter().rev() {
        // A property is "custom" if it is entirely non-numeric
        if key.to_utf8_lossy().parse::<usize>().is_err() {
            has_custom_properties = true;
        }

        let prop_value = array.get(key, activation).unwrap_or(Value::Undefined);
        let value = serialize(activation, prop_value);
        associative.push(Element::new(key.to_string(), Rc::new(value)));
    }
    if has_custom_properties {
        // Mixed Array: Has true non-numeric keys
        AmfValue::ECMAArray(ObjectId::INVALID, Vec::new(), associative, length as u32)
    } else {
        // Pure Dense Array: Pad holes with Undefined to maintain contiguous indices
        let mut dense = Vec::with_capacity(length);
        for i in 0..length {
            let elem_name = AvmString::new_utf8(activation.gc(), i.to_string());
            let prop_value = array.get(elem_name, activation).unwrap_or(Value::Undefined);
            let value = serialize(activation, prop_value);
            dense.push(Rc::new(value));
        }
        // Output as a StrictArray.
        AmfValue::StrictArray(ObjectId::INVALID, dense)
    }
}

/// Helper to serialize a specific value into the ObjWriter cache architecture.
fn serialize_value_to_writer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &str,
    elem: Value<'gc>,
    writer: &mut dyn ObjWriter<'_>,
) {
    match elem {
        Value::Object(o) => {
            if o.as_function().is_some() {
                // Flash entirely skips functions during object serialization
            } else if o.as_display_object().is_some() {
                writer.undefined(name)
            } else if let NativeObject::Array(_) = o.native() {
                let (aw, token) = writer.array(CacheKey::from_ptr(o.as_ptr()));

                if let Some(mut aw) = aw {
                    recursive_serialize(activation, o, &mut aw);

                    // TODO: What happens if an exception is thrown here?
                    let length = o
                        .length(activation)
                        .expect("Failed to get length for SharedObject array");

                    aw.commit(name, length as u32);
                } else {
                    writer.reference(name, token);
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
                let (ow, token) = writer.object(CacheKey::from_ptr(o.as_ptr()));

                if let Some(mut ow) = ow {
                    recursive_serialize(activation, o, &mut ow);
                    ow.commit(name);
                } else {
                    writer.reference(name, token);
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
) {
    // Reversed to match flash player ordering
    // Note that because get_keys can recurse, this may result in an OS stack overflow
    // This is still better than Flash's behavior, which results in a memory leak/crash
    for element_name in obj.get_keys(activation, false).into_iter().rev() {
        let elem = if obj.is_property_virtual(activation, element_name) {
            // Flash never evaluates getters during AMF serialization;
            // it serializes them as `undefined` (0x06).
            Value::Undefined
        } else {
            // Not a getter, safe to retrieve
            match obj.get(element_name, activation) {
                Ok(val) => val,
                Err(_) => continue,
            }
        };

        let name = element_name.to_utf8_lossy();
        serialize_value_to_writer(activation, name.as_ref(), elem, writer);
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

            for (i, item) in values.iter().enumerate() {
                let value = deserialize_value(activation, item, lso, reference_cache);
                obj.set_element(activation, i as i32, value).unwrap();
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
