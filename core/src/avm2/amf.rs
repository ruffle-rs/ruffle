use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::{ByteArrayObject, TObject};
use crate::avm2::ArrayObject;
use crate::avm2::ArrayStorage;
use crate::avm2::Multiname;
use crate::avm2::{Activation, Error, Object, Value};
use crate::string::AvmString;
use enumset::EnumSet;
use flash_lso::types::{Attribute, ClassDefinition, Value as AmfValue};
use flash_lso::types::{Element, Lso};

/// Serialize a Value to an AmfValue
fn serialize_value<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    elem: Value<'gc>,
) -> Option<AmfValue> {
    match elem {
        Value::Undefined => Some(AmfValue::Undefined),
        Value::Null => Some(AmfValue::Null),
        Value::Bool(b) => Some(AmfValue::Bool(b)),
        Value::Number(f) => Some(AmfValue::Number(f)),
        Value::Integer(num) => {
            // NOTE - we should really be converting `Value::Integer` to `Value::Number`
            // whenever it's outside this range, instead of performing this during AMF serialization.
            if num >= (1 << 28) || num < -(1 << 28) {
                Some(AmfValue::Number(num as f64))
            } else {
                Some(AmfValue::Integer(num))
            }
        }
        Value::String(s) => Some(AmfValue::String(s.to_string())),
        Value::Object(o) => {
            // TODO: Find a more general rule for which object types should be skipped,
            // and which turn into undefined.
            if o.as_executable().is_some() {
                None
            } else if o.as_display_object().is_some() {
                Some(AmfValue::Undefined)
            } else if let Some(array) = o.as_array_storage() {
                let mut values = Vec::new();
                recursive_serialize(activation, o, &mut values).unwrap();

                let mut dense = vec![];
                let mut sparse = vec![];
                for (i, elem) in (0..array.length()).zip(values.into_iter()) {
                    if elem.name == i.to_string() {
                        dense.push(elem.value.clone());
                    } else {
                        sparse.push(elem);
                    }
                }

                if sparse.is_empty() {
                    Some(AmfValue::StrictArray(dense))
                } else {
                    let len = sparse.len() as u32;
                    Some(AmfValue::ECMAArray(dense, sparse, len))
                }
            } else if let Some(date) = o.as_date_object() {
                date.date_time()
                    .map(|date_time| AmfValue::Date(date_time.timestamp_millis() as f64, None))
            } else {
                let is_object = o
                    .instance_of()
                    .map_or(false, |c| c == activation.avm2().classes().object);
                if is_object {
                    let mut object_body = Vec::new();
                    recursive_serialize(activation, o, &mut object_body).unwrap();
                    Some(AmfValue::Object(
                        object_body,
                        Some(ClassDefinition {
                            name: "".to_string(),
                            attributes: EnumSet::only(Attribute::Dynamic),
                            static_properties: Vec::new(),
                        }),
                    ))
                } else {
                    log::warn!(
                        "Serialization is not implemented for class other than Object: {:?}",
                        o
                    );
                    None
                }
            }
        }
    }
}

/// Serialize an Object and any children to a AMF object
pub fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    obj: Object<'gc>,
    elements: &mut Vec<Element>,
) -> Result<(), Error<'gc>> {
    let mut last_index = obj.get_next_enumerant(0, activation)?;
    while let Some(index) = last_index {
        let name = obj
            .get_enumerant_name(index, activation)?
            .coerce_to_string(activation)?;
        let value = obj.get_property(&Multiname::public(name), activation)?;

        if let Some(value) = serialize_value(activation, value) {
            elements.push(Element::new(name.to_utf8_lossy(), value));
        }
        last_index = obj.get_next_enumerant(index, activation)?;
    }
    Ok(())
}

/// Deserialize a AmfValue to a Value
pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    val: &AmfValue,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::Integer(num) => (*num).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.context.gc_context, s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ByteArray(bytes) => {
            let storage = ByteArrayStorage::from_vec(bytes.clone());
            let bytearray = ByteArrayObject::from_storage(activation, storage)?;
            bytearray.into()
        }
        AmfValue::ECMAArray(values, elements, _) => {
            // First let's create an array out of `values` (dense portion), then we add the elements onto it.
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value(activation, value)?));
            }
            let storage = ArrayStorage::from_storage(arr);
            let mut array = ArrayObject::from_storage(activation, storage)?;
            // Now let's add each element as a property
            for element in elements {
                array.set_property(
                    &Multiname::public(AvmString::new_utf8(
                        activation.context.gc_context,
                        element.name(),
                    )),
                    deserialize_value(activation, element.value())?,
                    activation,
                )?;
            }
            array.into()
        }
        AmfValue::StrictArray(values) => {
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value(activation, value)?));
            }
            let storage = ArrayStorage::from_storage(arr);
            let array = ArrayObject::from_storage(activation, storage)?;
            array.into()
        }
        AmfValue::Object(elements, class) => {
            if let Some(class) = class {
                if !class.name.is_empty() && class.name != "Object" {
                    log::warn!("Deserializing class {:?} is not supported!", class);
                }
            }

            let mut obj = activation
                .avm2()
                .classes()
                .object
                .construct(activation, &[])?;
            for entry in elements {
                let value = deserialize_value(activation, entry.value())?;
                obj.set_property(
                    &Multiname::public(AvmString::new_utf8(
                        activation.context.gc_context,
                        entry.name(),
                    )),
                    value,
                    activation,
                )?;
            }
            obj.into()
        }
        AmfValue::Date(time, _) => activation
            .avm2()
            .classes()
            .date
            .construct(activation, &[(*time).into()])?
            .into(),
        AmfValue::XML(content, _) => activation
            .avm2()
            .classes()
            .xml
            .construct(
                activation,
                &[Value::String(AvmString::new_utf8(
                    activation.context.gc_context,
                    content,
                ))],
            )?
            .into(),
        AmfValue::VectorDouble(..)
        | AmfValue::VectorUInt(..)
        | AmfValue::VectorInt(..)
        | AmfValue::VectorObject(..)
        | AmfValue::Dictionary(..)
        | AmfValue::Custom(..) => {
            log::error!("Deserialization not yet implemented: {:?}", val);
            Value::Undefined
        }
        AmfValue::AMF3(val) => deserialize_value(activation, val)?,
        AmfValue::Unsupported => Value::Undefined,
    })
}

/// Deserializes a Lso into an object containing the properties stored
pub fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    lso: &Lso,
) -> Result<Object<'gc>, Error<'gc>> {
    let mut obj = activation
        .avm2()
        .classes()
        .object
        .construct(activation, &[])?;

    for child in &lso.body {
        obj.set_property(
            &Multiname::public(AvmString::new_utf8(
                activation.context.gc_context,
                &child.name,
            )),
            deserialize_value(activation, child.value())?,
            activation,
        )?;
    }

    Ok(obj)
}
