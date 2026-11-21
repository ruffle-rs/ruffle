//! LocalConnection-specific AMF0 deserialization behavior.

use super::deserialization::{
    Amf0ArrayEntry, Amf0ReferenceCache, DeserializationContext, MAX_AMF0_REFERENCES,
    deserialize_array_entries, deserialize_object_entries, deserialize_value_inner,
};
use crate::avm1::globals::date::DeferredAmf0Date;
use crate::avm1::{Activation, ArrayBuilder, Object, Value};
use flash_lso::types::Value as AmfValue;

/// Count the referenceable values contained in an AMF0 value, including the
/// value itself when it is a complex type.
fn count_amf0_references(value: &AmfValue) -> usize {
    let count = match value {
        AmfValue::Object { data, .. } => 1usize.saturating_add(
            data.elements
                .iter()
                .map(|element| count_amf0_references(element.value()))
                .fold(0, usize::saturating_add),
        ),
        AmfValue::ECMAArray { data, .. } => {
            let dense_count = data
                .dense
                .iter()
                .map(count_amf0_references)
                .fold(0, usize::saturating_add);

            let element_count = data
                .elements
                .iter()
                .map(|element| count_amf0_references(element.value()))
                .fold(0, usize::saturating_add);

            1usize
                .saturating_add(dense_count)
                .saturating_add(element_count)
        }
        AmfValue::StrictArray { values, .. } => 1usize.saturating_add(
            values
                .iter()
                .map(count_amf0_references)
                .fold(0, usize::saturating_add),
        ),
        AmfValue::Date { .. } | AmfValue::XML { .. } => 1,
        _ => 0,
    };

    count.min(MAX_AMF0_REFERENCES)
}

/// Tracks the reference-table positions of values preceding the current
/// LocalConnection property. Earlier sibling values keep their wire indices,
/// but Flash does not preserve their identity when a later property refers to them.
pub(super) struct LocalConnectionReferenceState {
    references_before_property: usize,
}

impl LocalConnectionReferenceState {
    pub(super) fn new() -> Self {
        Self {
            // The top-level argument occupies reference index zero.
            references_before_property: 1,
        }
    }

    /// Create a property-local cache, preserving earlier indices as placeholders.
    pub(super) fn cache_for_property<'gc>(&self, root: Value<'gc>) -> Amf0ReferenceCache<'gc> {
        let mut reference_cache = Amf0ReferenceCache::default();
        reference_cache.insert(root);
        for _ in 1..self.references_before_property.min(MAX_AMF0_REFERENCES) {
            reference_cache.insert(Value::Undefined);
        }
        reference_cache
    }

    /// Advance past a property while retaining its wire-format reference count.
    pub(super) fn advance(&mut self, value: &AmfValue) {
        self.references_before_property = self
            .references_before_property
            .saturating_add(count_amf0_references(value))
            .min(MAX_AMF0_REFERENCES);
    }
}

/// Populate a LocalConnection array while retaining its per-sibling reference behavior.
fn deserialize_local_connection_array<'gc, 'a>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    entries: impl Iterator<Item = Amf0ArrayEntry<'a>>,
    deserialization_context: DeserializationContext,
    deferred_dates: &mut Vec<DeferredAmf0Date<'gc>>,
) -> Value<'gc> {
    let root: Value<'gc> = obj.into();
    let mut reference_state = LocalConnectionReferenceState::new();

    deserialize_array_entries(activation, obj, entries, |activation, value| {
        let mut reference_cache = reference_state.cache_for_property(root);
        let child = deserialize_value_inner(
            activation,
            value,
            &mut reference_cache,
            deserialization_context,
            deferred_dates,
        );
        reference_state.advance(value);
        child
    })
}

/// Create a LocalConnection array, even when the script-visible Array binding is absent.
fn create_local_connection_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    length: i32,
) -> Object<'gc> {
    // Flash's LocalConnection decoder does not invoke the script-visible Array
    // constructor, which may have been replaced or made to throw.
    ArrayBuilder::empty_with_len(activation, length)
}

/// Deserialize a LocalConnection argument. Reference indices are still counted
/// across the argument, but values from earlier sibling properties are not
/// carried into a later property's cache.
pub(crate) fn deserialize_local_connection_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    deserialization_context: DeserializationContext,
    deferred_dates: &mut Vec<DeferredAmf0Date<'gc>>,
) -> Value<'gc> {
    match val {
        AmfValue::Object { data, .. } => {
            let obj = Object::new(
                &activation.context.strings,
                Some(activation.prototypes().object),
            );
            let root: Value<'gc> = obj.into();
            let mut reference_state = LocalConnectionReferenceState::new();

            deserialize_object_entries(
                activation,
                obj,
                data.elements.iter(),
                |activation, value| {
                    let mut reference_cache = reference_state.cache_for_property(root);
                    let child = deserialize_value_inner(
                        activation,
                        value,
                        &mut reference_cache,
                        deserialization_context,
                        deferred_dates,
                    );
                    reference_state.advance(value);
                    child
                },
            )
        }
        AmfValue::ECMAArray { data, .. } => {
            let obj = create_local_connection_array(
                activation,
                i32::try_from(data.length).unwrap_or(i32::MAX),
            );

            let dense_entries = data
                .dense
                .iter()
                .enumerate()
                .map(|(index, value)| Amf0ArrayEntry::Dense { index, value });
            let property_entries = data.elements.iter().map(Amf0ArrayEntry::Property);

            deserialize_local_connection_array(
                activation,
                obj,
                dense_entries.chain(property_entries),
                deserialization_context,
                deferred_dates,
            )
        }
        AmfValue::StrictArray { values, .. } => {
            let obj = create_local_connection_array(
                activation,
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            );

            let entries = values
                .iter()
                .enumerate()
                .map(|(index, value)| Amf0ArrayEntry::Dense { index, value });

            deserialize_local_connection_array(
                activation,
                obj,
                entries,
                deserialization_context,
                deferred_dates,
            )
        }
        _ => {
            let mut reference_cache = Amf0ReferenceCache::default();
            deserialize_value_inner(
                activation,
                val,
                &mut reference_cache,
                deserialization_context,
                deferred_dates,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::error::Error;
    use crate::avm1::test_utils::with_avm;
    use crate::string::AvmString;
    use flash_lso::types::{DateValue, ECMAArrayObjectValue, Element, ObjectId, ObjectValue};

    #[test]
    fn local_connection_reference_state_keeps_sibling_indices_as_placeholders() {
        let root = Value::Null;
        let mut state = LocalConnectionReferenceState::new();

        assert_eq!(state.cache_for_property(root).values, vec![root]);

        let previous_property = AmfValue::Date {
            id: ObjectId::INVALID,
            data: DateValue {
                time: 0.0,
                timezone_or_utc: None,
            },
        };
        state.advance(&previous_property);

        let next_property_cache = state.cache_for_property(root);
        assert_eq!(next_property_cache.values, vec![root, Value::Undefined]);
    }

    #[test]
    fn local_connection_deserializes_strict_array_values() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let value = AmfValue::StrictArray {
                id: ObjectId::INVALID,
                values: vec![AmfValue::Number(42.0)],
            };

            let mut deferred_dates = Vec::new();
            let array = deserialize_local_connection_value(
                activation,
                &value,
                DeserializationContext::LocalConnection,
                &mut deferred_dates,
            )
            .as_object(activation)
            .expect("LocalConnection strict array should deserialize to an array");

            assert_eq!(array.get_element(activation, 0), 42.0.into());

            Ok(())
        });
    }

    #[test]
    fn local_connection_deserializes_dense_ecma_array_values() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let value = AmfValue::ECMAArray {
                id: ObjectId::INVALID,
                data: ECMAArrayObjectValue {
                    dense: vec![
                        AmfValue::Object {
                            id: ObjectId::INVALID,
                            data: ObjectValue {
                                elements: vec![Element::new(
                                    "marker",
                                    AmfValue::String("dense".to_string()),
                                )],
                                class_definition: None,
                            },
                        },
                        AmfValue::Number(42.0),
                    ],
                    elements: Vec::new(),
                    length: 2,
                },
            };

            let mut deferred_dates = Vec::new();
            let array = deserialize_local_connection_value(
                activation,
                &value,
                DeserializationContext::LocalConnection,
                &mut deferred_dates,
            )
            .as_object(activation)
            .expect("LocalConnection ECMA array should deserialize to an array");

            let Value::Object(first) = array.get_element(activation, 0) else {
                panic!("First dense ECMA array value should be an object");
            };
            assert_eq!(
                first.get(AvmString::new_utf8(activation.gc(), "marker"), activation)?,
                Value::String(AvmString::new_utf8(activation.gc(), "dense")),
            );
            assert_eq!(array.get_element(activation, 1), 42.0.into());

            Ok(())
        });
    }
}
