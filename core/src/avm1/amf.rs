const MAX_AMF0_REFERENCES: usize = u16::MAX as usize + 1;

mod deserialization;
mod local_connection;
mod serialization;

pub(crate) use deserialization::{Amf0ReferenceCache, DeserializationContext, deserialize_value};
pub(crate) use local_connection::deserialize_local_connection_value;
pub(crate) use serialization::{SerializationContext, recursive_serialize, serialize_with_context};

#[cfg(test)]
mod tests {
    use super::local_connection::LocalConnectionReferenceState;
    use super::*;
    use crate::avm1::Value;
    use crate::avm1::error::Error;
    use crate::avm1::test_utils::with_avm;
    use crate::string::AvmString;
    use flash_lso::types::{
        DateValue, ECMAArrayObjectValue, Element, ObjectId, ObjectValue, Value as AmfValue,
    };

    #[test]
    fn reference_cache_stops_at_the_amf0_reference_limit() {
        with_avm(8, |_activation, _this| -> Result<(), Error> {
            let mut reference_cache = Amf0ReferenceCache::default();

            // AMF0 references use a 16-bit index, so indices 0 through 65535
            // are the complete set of representable references.
            for _ in 0..=u16::MAX {
                reference_cache.insert(Value::Undefined);
            }
            assert_eq!(reference_cache.values.len(), MAX_AMF0_REFERENCES);

            // Additional objects cannot be addressed by an AMF0 reference.
            reference_cache.insert(Value::Null);
            assert_eq!(reference_cache.values.len(), MAX_AMF0_REFERENCES);

            Ok(())
        });
    }

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
