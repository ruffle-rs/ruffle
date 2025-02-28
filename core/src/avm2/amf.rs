use std::collections::BTreeMap;
use std::rc::Rc;

use super::property::Property;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::object::{ByteArrayObject, ClassObject, ScriptObject, TObject, VectorObject};
use crate::avm2::vector::VectorStorage;
use crate::avm2::ArrayObject;
use crate::avm2::ArrayStorage;
use crate::avm2::{Activation, Error, Object, Value};
use crate::avm2_stub_method;
use crate::string::AvmString;
use enumset::EnumSet;
use flash_lso::types::{AMFVersion, Element, Lso, ObjectId};
use flash_lso::types::{Attribute, ClassDefinition, Value as AmfValue};
use fnv::FnvHashMap;

pub type ObjectTable<'gc> = FnvHashMap<Object<'gc>, Rc<AmfValue>>;

/// Serialize a Value to an AmfValue
pub fn serialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    elem: Value<'gc>,
    amf_version: AMFVersion,
    object_table: &mut ObjectTable<'gc>,
) -> Option<AmfValue> {
    match elem {
        Value::Undefined => Some(AmfValue::Undefined),
        Value::Null => Some(AmfValue::Null),
        Value::Bool(b) => Some(AmfValue::Bool(b)),
        Value::Number(f) => Some(AmfValue::Number(f)),
        Value::Integer(num) => {
            // NOTE - we should really be converting `Value::Integer` to `Value::Number`
            // whenever it's outside this range, instead of performing this during AMF serialization.
            // Integers are unsupported in AMF0, and must be converted to Number regardless of whether
            // it can be represented as an integer.
            // FIXME - handle coercion floats like '1.0' to integers
            if amf_version == AMFVersion::AMF0 || num >= (1 << 28) || num < -(1 << 28) {
                Some(AmfValue::Number(num as f64))
            } else {
                Some(AmfValue::Integer(num))
            }
        }
        Value::String(s) => Some(AmfValue::String(s.to_string())),
        Value::Object(o) => {
            // TODO: Find a more general rule for which object types should be skipped,
            // and which turn into undefined.
            if o.as_function_object().is_some() {
                None
            } else if o.as_display_object().is_some() {
                Some(AmfValue::Undefined)
            } else if let Some(array_storage) = o.as_array_storage() {
                let len = array_storage.length() as u32;
                drop(array_storage);

                let mut values = Vec::new();
                // Don't serialize properties from the vtable (we don't want a 'length' field)
                recursive_serialize(activation, o, &mut values, None, amf_version, object_table)
                    .unwrap();

                if amf_version == AMFVersion::AMF3 {
                    let mut dense = vec![];
                    let mut sparse = vec![];
                    // ActionScript `Array`s can have non-number properties, and these properties
                    // are confirmed and tested to also be serialized, so do not limit the values
                    // iterated over by the length of the internal array data.
                    for (i, elem) in values.into_iter().enumerate() {
                        if elem.name == i.to_string() {
                            dense.push(elem.value.clone());
                        } else {
                            sparse.push(elem);
                        }
                    }

                    Some(AmfValue::ECMAArray(ObjectId::INVALID, dense, sparse, len))
                } else {
                    // TODO: is this right?
                    Some(AmfValue::ECMAArray(ObjectId::INVALID, vec![], values, len))
                }
            } else if let Some(vec) = o.as_vector_storage() {
                let val_type = vec.value_type();
                if val_type == Some(activation.avm2().class_defs().int) {
                    let int_vec: Vec<_> = vec.iter().map(|v| v.as_i32()).collect();
                    Some(AmfValue::VectorInt(int_vec, vec.is_fixed()))
                } else if val_type == Some(activation.avm2().class_defs().uint) {
                    let uint_vec: Vec<_> = vec.iter().map(|v| v.as_u32()).collect();
                    Some(AmfValue::VectorUInt(uint_vec, vec.is_fixed()))
                } else if val_type == Some(activation.avm2().class_defs().number) {
                    let num_vec: Vec<_> = vec.iter().map(|v| v.as_f64()).collect();
                    Some(AmfValue::VectorDouble(num_vec, vec.is_fixed()))
                } else {
                    let obj_vec: Vec<_> = vec
                        .iter()
                        .map(|v| {
                            serialize_value(activation, v, amf_version, object_table)
                                .unwrap_or(AmfValue::Undefined)
                        })
                        .collect();

                    let val_type = val_type.unwrap_or(activation.avm2().class_defs().object);

                    let name = class_to_alias(activation, val_type);
                    Some(AmfValue::VectorObject(
                        ObjectId::INVALID,
                        obj_vec,
                        name,
                        vec.is_fixed(),
                    ))
                }
            } else if let Some(date) = o.as_date_object() {
                date.date_time()
                    .map(|date_time| AmfValue::Date(date_time.timestamp_millis() as f64, None))
            } else if let Some(xml) = o.as_xml_object() {
                // `is_string` is `true` for the AS3 XML class
                Some(AmfValue::XML(
                    xml.node().xml_to_xml_string(activation).to_string(),
                    true,
                ))
            } else if let Some(bytearray) = o.as_bytearray() {
                Some(AmfValue::ByteArray(bytearray.bytes().to_vec()))
            } else {
                let class = o.instance_class();
                let name = class_to_alias(activation, class);

                let mut attributes = EnumSet::empty();
                if !class.is_sealed() {
                    attributes.insert(Attribute::Dynamic);
                }

                let mut object_body = Vec::new();
                let mut static_properties = Vec::new();
                recursive_serialize(
                    activation,
                    o,
                    &mut object_body,
                    Some(&mut static_properties),
                    amf_version,
                    object_table,
                )
                .unwrap();
                Some(AmfValue::Object(
                    ObjectId::INVALID,
                    object_body,
                    if amf_version == AMFVersion::AMF3 {
                        Some(ClassDefinition {
                            name,
                            attributes,
                            // FIXME - implement this
                            static_properties,
                        })
                    } else {
                        None
                    },
                ))
            }
        }
    }
}

fn alias_to_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    alias: AvmString<'gc>,
) -> Result<ClassObject<'gc>, Error<'gc>> {
    if let Some(class_object) = activation.avm2().get_class_by_alias(alias) {
        Ok(class_object)
    } else {
        Ok(activation.avm2().classes().object)
    }
}

fn class_to_alias<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> String {
    if let Some(alias) = activation.avm2().get_alias_by_class(class) {
        alias.to_string()
    } else {
        "".to_string()
    }
}

/// Serialize an Object and any children to a AMF object
pub fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    elements: &mut Vec<Element>,
    static_properties: Option<&mut Vec<String>>,
    amf_version: AMFVersion,
    object_table: &mut ObjectTable<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(static_properties) = static_properties {
        let vtable = obj.vtable();
        // TODO: respect versioning
        let mut props = vtable.public_properties();
        // Flash appears to use vtable iteration order, but we sort ours
        // to make our test output consistent.
        props.sort_by_key(|(name, _)| name.to_utf8_lossy().to_string());
        for (name, prop) in props {
            if let Property::Method { .. } = prop {
                continue;
            }
            if let Property::Virtual { get, set } = prop {
                if !(get.is_some() && set.is_some()) {
                    continue;
                }
            }
            let value = Value::from(obj).get_public_property(name, activation)?;
            let name = name.to_utf8_lossy().to_string();
            if let Some(elem) =
                get_or_create_element(activation, name.clone(), value, object_table, amf_version)
            {
                elements.push(elem);
                static_properties.push(name);
            }
        }
    }

    // FIXME: Flash only seems to use this enumeration for dynamic classes.
    let mut last_index = obj.get_next_enumerant(0, activation)?;
    while last_index != 0 {
        let name = obj
            .get_enumerant_name(last_index, activation)?
            .coerce_to_string(activation)?;
        let value = obj.get_enumerant_value(last_index, activation)?;

        let name = name.to_utf8_lossy().to_string();
        if let Some(elem) =
            get_or_create_element(activation, name.clone(), value, object_table, amf_version)
        {
            elements.push(elem);
        }
        last_index = obj.get_next_enumerant(last_index, activation)?;
    }
    Ok(())
}

fn get_or_create_element<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: String,
    val: Value<'gc>,
    object_table: &mut ObjectTable<'gc>,
    amf_version: AMFVersion,
) -> Option<Element> {
    if let Some(obj) = val.as_object() {
        let rc_val = match object_table.get(&obj) {
            Some(rc_val) => {
                // Even though we'll clone the same 'Rc<AmfValue>' for each occurrence
                // of 'Object', flash_lso doesn't serialize this correctly yet.
                avm2_stub_method!(
                    activation,
                    "flash.utils.ByteArray",
                    "writeObject",
                    "with same Object used multiple times"
                );
                Some(rc_val.clone())
            }
            None => {
                if let Some(value) = serialize_value(activation, val, amf_version, object_table) {
                    let rc_val = Rc::new(value);
                    // We cannot use Entry, since we need to pass in 'object_table' to 'serialize_value'
                    object_table.insert(obj, rc_val.clone());
                    Some(rc_val)
                } else {
                    None
                }
            }
        };
        return rc_val.map(|val| Element::new(name, val));
    } else if let Some(value) = serialize_value(activation, val, amf_version, object_table) {
        return Some(Element::new(name, Rc::new(value)));
    }
    None
}

/// Deserialize a AmfValue to a Value
pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
) -> Result<Value<'gc>, Error<'gc>> {
    let mut x = BTreeMap::new();
    deserialize_value_impl(activation, val, &mut x)
}

pub fn deserialize_value_impl<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    object_map: &mut BTreeMap<ObjectId, Object<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::Integer(num) => (*num).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.gc(), s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ByteArray(bytes) => {
            let storage = ByteArrayStorage::from_vec(bytes.clone());
            let bytearray = ByteArrayObject::from_storage(activation, storage)?;
            bytearray.into()
        }
        AmfValue::ECMAArray(id, values, elements, _) => {
            let empty_storage = ArrayStorage::new(0);
            let array = ArrayObject::from_storage(activation, empty_storage);
            object_map.insert(*id, array.into());

            // First let's create an array out of `values` (dense portion), then we add the elements onto it.
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value_impl(activation, value, object_map)?));
            }
            array
                .array_storage_mut(activation.gc())
                .replace_dense_storage(arr);

            // Now let's add each element as a property
            for element in elements {
                array.set_string_property_local(
                    AvmString::new_utf8(activation.gc(), element.name()),
                    deserialize_value_impl(activation, element.value(), object_map)?,
                    activation,
                )?;
            }
            array.into()
        }
        AmfValue::StrictArray(id, values) => {
            let empty_storage = ArrayStorage::new(0);
            let array = ArrayObject::from_storage(activation, empty_storage);
            object_map.insert(*id, array.into());

            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value_impl(activation, value, object_map)?));
            }

            array
                .array_storage_mut(activation.gc())
                .replace_dense_storage(arr);

            array.into()
        }
        AmfValue::Object(id, elements, class) => {
            let target_class = if let Some(class) = class {
                let name = AvmString::new_utf8(activation.gc(), &class.name);
                alias_to_class(activation, name)?
            } else {
                activation.avm2().classes().object
            };
            let obj = target_class.construct(activation, &[])?;
            object_map.insert(*id, obj.as_object().unwrap());

            for entry in elements {
                let name = entry.name();
                let value = deserialize_value_impl(activation, entry.value(), object_map)?;
                // Flash player logs the error and continues deserializing the rest of the object,
                // even when calling a customer setter
                if let Err(e) = obj.set_public_property(
                    AvmString::new_utf8(activation.gc(), name),
                    value,
                    activation,
                ) {
                    tracing::warn!(
                        "Ignoring error deserializing AMF property for field {name:?}: {e:?}"
                    );
                    if let Error::AvmError(e) = e {
                        if let Some(e) = e.as_object().and_then(|o| o.as_error_object()) {
                            // Flash player *traces* the error (without a stacktrace)
                            activation.context.avm_trace(&e.display().to_string());
                        }
                    }
                }
            }

            obj
        }
        AmfValue::Date(time, _) => activation
            .avm2()
            .classes()
            .date
            .construct(activation, &[(*time).into()])?,
        AmfValue::XML(content, _) => activation.avm2().classes().xml.construct(
            activation,
            &[Value::String(AvmString::new_utf8(activation.gc(), content))],
        )?,
        AmfValue::VectorDouble(vec, is_fixed) => {
            let storage = VectorStorage::from_values(
                vec.iter().map(|v| (*v).into()).collect(),
                *is_fixed,
                Some(activation.avm2().class_defs().number),
            );
            VectorObject::from_vector(storage, activation)?.into()
        }
        AmfValue::VectorUInt(vec, is_fixed) => {
            let storage = VectorStorage::from_values(
                vec.iter().map(|v| (*v).into()).collect(),
                *is_fixed,
                Some(activation.avm2().class_defs().uint),
            );
            VectorObject::from_vector(storage, activation)?.into()
        }
        AmfValue::VectorInt(vec, is_fixed) => {
            let storage = VectorStorage::from_values(
                vec.iter().map(|v| (*v).into()).collect(),
                *is_fixed,
                Some(activation.avm2().class_defs().int),
            );
            VectorObject::from_vector(storage, activation)?.into()
        }
        AmfValue::VectorObject(id, vec, ty_name, is_fixed) => {
            let name = AvmString::new_utf8(activation.gc(), ty_name);
            let class = alias_to_class(activation, name)?;

            // Create an empty vector, as it has to exist in the map before reading children, in case they reference it
            let empty_storage = VectorStorage::new(
                0,
                *is_fixed,
                Some(class.inner_class_definition()),
                activation,
            );
            let obj = VectorObject::from_vector(empty_storage, activation)?;
            object_map.insert(*id, obj);

            let new_values = vec
                .iter()
                .map(|v| {
                    deserialize_value_impl(activation, v, object_map).map(|value| {
                        // There's no Vector.<void>: convert any
                        // Undefined items in the Vector to Null.
                        if matches!(value, Value::Undefined) {
                            Value::Null
                        } else {
                            value
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Swap in the actual values
            obj.as_vector_storage_mut(activation.gc())
                .expect("Failed to get vector storage from VectorObject")
                .replace_storage(new_values);

            obj.into()
        }
        AmfValue::Dictionary(id, values, has_weak_keys) => {
            let obj = activation
                .avm2()
                .classes()
                .dictionary
                .construct(activation, &[(*has_weak_keys).into()])?
                .as_object()
                .unwrap();

            object_map.insert(*id, obj);
            let dict_obj = obj
                .as_dictionary_object()
                .expect("Failed to get dictionary from constructed object");

            for (key, value) in values {
                let key = deserialize_value_impl(activation, key, object_map)?;
                let value = deserialize_value_impl(activation, value, object_map)?;

                if let Value::Object(key) = key {
                    dict_obj.set_property_by_object(key, value, activation.gc());
                } else {
                    let key_string = key.coerce_to_string(activation)?;
                    dict_obj.set_string_property_local(key_string, value, activation)?;
                }
            }
            dict_obj.into()
        }
        AmfValue::Custom(..) => {
            tracing::error!("Deserialization not yet implemented for Custom: {:?}", val);
            Value::Undefined
        }
        AmfValue::Reference(_) => {
            tracing::error!(
                "Deserialization not yet implemented for Reference: {:?}",
                val
            );
            Value::Undefined
        }
        AmfValue::AMF3(val) => deserialize_value_impl(activation, val, object_map)?,
        AmfValue::Unsupported => Value::Undefined,
        AmfValue::Amf3ObjectReference(r) => {
            if let Some(o) = object_map.get(r) {
                (*o).into()
            } else {
                tracing::error!("AMF3 deserializer got an object reference {r:?} to an object we've not seen yet");
                Value::Undefined
            }
        }
    })
}

/// Deserializes a Lso into an object containing the properties stored
pub fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    lso: &Lso,
) -> Result<Object<'gc>, Error<'gc>> {
    let obj = ScriptObject::new_object(activation);

    for child in &lso.body {
        obj.set_string_property_local(
            AvmString::new_utf8(activation.gc(), &child.name),
            deserialize_value(activation, child.value())?,
            activation,
        )?;
    }

    Ok(obj)
}
