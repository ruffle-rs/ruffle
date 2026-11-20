const MAX_AMF0_REFERENCES: usize = u16::MAX as usize + 1;

mod deserialization;
mod local_connection;
mod serialization;

pub(crate) use deserialization::{Amf0ReferenceCache, DeserializationContext, deserialize_value};
pub(crate) use local_connection::deserialize_local_connection_value;
pub(crate) use serialization::{SerializationContext, recursive_serialize, serialize_with_context};
