//! Shared AMF0 value deserialization.

use super::MAX_AMF0_REFERENCES;
use crate::avm1::globals::date::DeferredAmf0Date;
use crate::avm1::globals::date::from_amf0 as date_from_amf0;
use crate::avm1::globals::date::from_local_connection_amf0 as local_date_from_amf0;
use crate::avm1::globals::xml::from_amf0 as xml_from_amf0;
use crate::avm1::object::parse_canonical_array_index;
use crate::avm1::{Activation, ArrayBuilder, Attribute, Object, Value};
use crate::string::{AvmString, from_utf8};
use flash_lso::types::{Element, Reference, Value as AmfValue};

/// Tracks AMF0 referenceable values in wire order.
#[derive(Default)]
pub(crate) struct Amf0ReferenceCache<'gc> {
    pub(super) values: Vec<Value<'gc>>,
}

impl<'gc> Amf0ReferenceCache<'gc> {
    pub(crate) fn insert(&mut self, value: Value<'gc>) {
        // AMF0 reference indices are unsigned 16-bit integers.
        if self.values.len() < MAX_AMF0_REFERENCES {
            self.values.push(value);
        }
    }

    fn get(&self, reference: Reference) -> Option<Value<'gc>> {
        self.values.get(usize::from(reference.index())).copied()
    }
}

/// Selects the Flash-specific behavior used when deserializing an AMF0 value.
#[derive(Clone, Copy)]
pub(crate) enum DeserializationContext {
    /// NetConnection remoting: XML values participate in the reference table.
    NetConnection,
    /// LocalConnection: Date construction is deferred and XML is not referenceable.
    LocalConnection,
    /// SharedObject LSO: XML values do not participate in the reference table.
    SharedObject,
}

/// Deserialize an AMF0 value using the transport's reference-table semantics.
pub(crate) fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
    deserialization_context: DeserializationContext,
) -> Value<'gc> {
    let mut deferred_dates = Vec::new();
    deserialize_value_inner(
        activation,
        val,
        reference_cache,
        deserialization_context,
        &mut deferred_dates,
    )
}

pub(super) fn deserialize_value_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
    deserialization_context: DeserializationContext,
    deferred_dates: &mut Vec<DeferredAmf0Date<'gc>>,
) -> Value<'gc> {
    match val {
        AmfValue::Null => Value::Null,
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Number(f) => (*f).into(),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.gc(), s)),
        AmfValue::Bool(b) => (*b).into(),
        AmfValue::ECMAArray { data, .. } => {
            let obj = ArrayBuilder::empty_with_len(
                activation,
                i32::try_from(data.length).unwrap_or(i32::MAX),
            );

            // Register before deserializing children so references back to this array work.
            reference_cache.insert(obj.into());

            let entries = data.elements.iter().map(Amf0ArrayEntry::Property);
            deserialize_array_entries(activation, obj, entries, |activation, value| {
                deserialize_value_inner(
                    activation,
                    value,
                    reference_cache,
                    deserialization_context,
                    deferred_dates,
                )
            })
        }
        AmfValue::StrictArray { values, .. } => {
            let obj = ArrayBuilder::empty_with_len(
                activation,
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            );

            // Register before deserializing children so self-references work.
            reference_cache.insert(obj.into());

            let entries = values
                .iter()
                .enumerate()
                .map(|(index, value)| Amf0ArrayEntry::Dense { index, value });
            deserialize_array_entries(activation, obj, entries, |activation, value| {
                deserialize_value_inner(
                    activation,
                    value,
                    reference_cache,
                    deserialization_context,
                    deferred_dates,
                )
            })
        }
        AmfValue::Object { data, .. } => {
            let obj = Object::new(
                &activation.context.strings,
                Some(activation.prototypes().object),
            );

            let value: Value<'gc> = obj.into();

            // Register before deserializing children so self-references work.
            reference_cache.insert(value);

            deserialize_object_entries(
                activation,
                obj,
                data.elements.iter(),
                |activation, value| {
                    deserialize_value_inner(
                        activation,
                        value,
                        reference_cache,
                        deserialization_context,
                        deferred_dates,
                    )
                },
            )
        }
        AmfValue::Date { data, .. } => {
            let value = match deserialization_context {
                DeserializationContext::LocalConnection => {
                    local_date_from_amf0(activation, data.time, deferred_dates)
                }
                DeserializationContext::NetConnection | DeserializationContext::SharedObject => {
                    date_from_amf0(activation, data.time)
                }
            };

            reference_cache.insert(value);
            value
        }
        AmfValue::XML { value: content, .. } => {
            // Flash creates a native XML document using the current XML prototype,
            // without invoking the script-visible XML constructor.
            let value = xml_from_amf0(activation, content);

            if matches!(
                deserialization_context,
                DeserializationContext::NetConnection
            ) {
                reference_cache.insert(value);
            }
            value
        }
        AmfValue::Reference(reference) => {
            // Invalid references in malformed files resolve to undefined.
            reference_cache.get(*reference).unwrap_or(Value::Undefined)
        }
        _ => Value::Undefined,
    }
}

/// One entry in an AMF0 array, in wire order.
pub(super) enum Amf0ArrayEntry<'a> {
    Dense { index: usize, value: &'a AmfValue },
    Property(&'a Element),
}

/// Where a decoded AMF0 array entry should be stored.
enum Amf0ArrayProperty<'a> {
    Index(i32),
    Named(&'a Element),
}

/// Populate an AVM1 array using the supplied AMF0 value decoder.
pub(super) fn deserialize_array_entries<'gc, 'a>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    entries: impl Iterator<Item = Amf0ArrayEntry<'a>>,
    mut deserialize_value: impl FnMut(&mut Activation<'_, 'gc>, &AmfValue) -> Value<'gc>,
) -> Value<'gc> {
    let root: Value<'gc> = obj.into();

    for entry in entries {
        let (value, property) = match entry {
            Amf0ArrayEntry::Dense { index, value } => {
                (value, Amf0ArrayProperty::Index(index as i32))
            }
            Amf0ArrayEntry::Property(entry) => {
                let property = match parse_canonical_array_index(from_utf8(entry.name()).as_ref()) {
                    Some(index) => Amf0ArrayProperty::Index(index),
                    None => Amf0ArrayProperty::Named(entry),
                };
                (entry.value(), property)
            }
        };

        let child = deserialize_value(activation, value);

        // Every supplied entry is explicit, even when its value is undefined.
        // Missing ECMA-array entries are not present in the iterator and remain holes.
        match property {
            Amf0ArrayProperty::Index(index) => {
                obj.set_element(activation, index, child)
                    .expect("Failed to set an AMF0 array element");
            }
            Amf0ArrayProperty::Named(entry) => {
                obj.define_value(
                    activation.gc(),
                    AvmString::new_utf8(activation.gc(), &entry.name),
                    child,
                    Attribute::empty(),
                );
            }
        }
    }

    root
}

/// Populate an AVM1 object using the supplied AMF0 value decoder.
pub(super) fn deserialize_object_entries<'gc, 'a>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    entries: impl Iterator<Item = &'a Element>,
    mut deserialize_value: impl FnMut(&mut Activation<'_, 'gc>, &AmfValue) -> Value<'gc>,
) -> Value<'gc> {
    let root: Value<'gc> = obj.into();

    for entry in entries {
        let child = deserialize_value(activation, entry.value());
        let name = AvmString::new_utf8(activation.gc(), &entry.name);
        obj.define_value(activation.gc(), name, child, Attribute::empty());
    }

    root
}
