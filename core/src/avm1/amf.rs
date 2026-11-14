use crate::avm1::globals::date::DeferredAmf0Date;
use crate::avm1::globals::date::from_amf0 as date_from_amf0;
use crate::avm1::globals::date::from_local_connection_amf0 as local_date_from_amf0;
use crate::avm1::globals::xml::from_amf0 as xml_from_amf0;
use crate::avm1::object::parse_canonical_array_index;
use crate::avm1::{Activation, ArrayBuilder, Attribute, NativeObject, Object, Value};
use crate::string::{AvmString, from_utf8};
use flash_lso::amf0::writer::{Amf0Writer, CacheKey, ObjWriter};
use flash_lso::types::{
    ClassDefinition, DateValue, ECMAArrayObjectValue, Element, ObjectId, ObjectValue, Reference,
    Value as AmfValue,
};
use ruffle_macros::istr;

/// Tracks AMF0 referenceable values in wire order.
#[derive(Default)]
pub(crate) struct Amf0ReferenceCache<'gc> {
    values: Vec<Value<'gc>>,
}

impl<'gc> Amf0ReferenceCache<'gc> {
    pub(crate) fn insert(&mut self, value: Value<'gc>) {
        // AMF0 reference indices are unsigned 16-bit integers.
        if self.values.len() <= usize::from(u16::MAX) {
            self.values.push(value);
        }
    }

    fn get(&self, reference: Reference) -> Option<Value<'gc>> {
        self.values.get(usize::from(reference.index())).copied()
    }
}

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

/// Serialize AMF data for LocalConnection.send.
pub fn serialize<'gc>(activation: &mut Activation<'_, 'gc>, value: Value<'gc>) -> AmfValue {
    serialize_with_array_encoding(activation, value, ArrayEncoding::StrictWhenDense)
}

/// Serialize AMF data for NetConnection.addHeader and NetConnection.call.
pub fn serialize_net_connection<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> AmfValue {
    serialize_with_array_encoding(
        activation,
        value,
        ArrayEncoding::StrictWhenNoCustomProperties,
    )
}

fn serialize_with_array_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
    array_encoding: ArrayEncoding,
) -> AmfValue {
    match value {
        Value::Undefined => AmfValue::Undefined,
        Value::Null => AmfValue::Null,
        Value::Bool(bool) => AmfValue::Bool(bool),
        Value::Number(number) => AmfValue::Number(number),
        Value::String(string) => AmfValue::String(string.to_string()),
        Value::Object(object) => {
            if object.as_display_object().is_some() {
                AmfValue::Undefined
            } else if object.is_native_array() {
                serialize_array(activation, object, array_encoding)
            } else if let Some(xml_node) = object.as_xml_node() {
                let string = xml_node
                    .into_string(activation)
                    .expect("Failed to convert xml to string in SharedObject");
                AmfValue::XML {
                    value: string.to_utf8_lossy().into_owned(),
                    is_string: true,
                }
            } else if let NativeObject::Date(date) = object.native() {
                // Flash resolves the global Date constructor while serializing AMF0 dates.
                let global = activation.global_object();
                let _ = global.get(istr!("Date"), activation);

                AmfValue::Date {
                    id: ObjectId::INVALID,
                    data: DateValue {
                        time: date.get().time(),
                        timezone_or_utc: None,
                    },
                }
            } else {
                let class_def =
                    object_class_name(activation, object).map(ClassDefinition::default_with_name);
                let elements = serialize_object_properties(activation, object, array_encoding);
                AmfValue::Object {
                    id: ObjectId::INVALID,
                    data: ObjectValue {
                        elements,
                        class_definition: class_def,
                    },
                }
            }
        }
        Value::MovieClip(_) => AmfValue::Undefined,
    }
}

/// Serialize an Object in NetConnection or LocalConnection.
fn serialize_object_properties<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
    array_encoding: ArrayEncoding,
) -> Vec<Element> {
    let mut w = Amf0Writer::default();

    // The root object is written by the enclosing NetConnection/LocalConnection
    // value, while this writer serializes only its properties. Reserve its slot
    // here so references in those properties use the wire-format index.
    let root_key = CacheKey::from_ptr(object.as_ptr());
    let root_reference = w.make_reference();
    w.cache_add(root_key, root_reference);

    recursive_serialize(activation, object, &mut w, array_encoding);
    w.commit_lso("root").into_iter().collect()
}

/// Parse a numeric array key the way Flash's NetConnection AMF serializer does.
/// Leading zeroes are accepted, but leading plus signs, whitespace, and negative
/// values remain associative property names.
fn net_connection_array_index(name: &str) -> Option<u32> {
    // Rust's integer parser accepts a leading '+', but Flash keeps that key associative.
    if name.starts_with('+') {
        None
    } else {
        name.parse::<u32>().ok()
    }
}

/// Serialize an array according to the transport-specific AMF0 array rules.
fn serialize_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    array: Object<'gc>,
    array_encoding: ArrayEncoding,
) -> AmfValue {
    let length = array.length(activation).unwrap_or(0).max(0) as usize;
    let keys = array.get_keys(activation, false);
    let array_info = array_serialization_info(&keys, length, array_encoding);

    if array_info.is_strict_array {
        let mut dense = Vec::with_capacity(array_info.length);
        for i in 0..array_info.length {
            let prop_value = array.get_element(activation, i as i32);
            dense.push(serialize_with_array_encoding(
                activation,
                prop_value,
                array_encoding,
            ));
        }
        AmfValue::StrictArray {
            id: ObjectId::INVALID,
            values: dense,
        }
    } else {
        // ECMA arrays preserve associative properties and, for LocalConnection,
        // preserve sparse indices without materializing undefined holes.
        let mut elements = Vec::with_capacity(keys.len());
        for key in keys.into_iter().rev() {
            let prop_value = array.get(key, activation).unwrap_or(Value::Undefined);
            elements.push(Element::new(
                key.to_string(),
                serialize_with_array_encoding(activation, prop_value, array_encoding),
            ));
        }
        AmfValue::ECMAArray {
            id: ObjectId::INVALID,
            data: ECMAArrayObjectValue {
                dense: Vec::new(),
                elements,
                length: array_info.length as u32,
            },
        }
    }
}

/// How native AVM1 arrays are represented in the enclosing AMF0 format.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayEncoding {
    /// SharedObject LSO bodies encode arrays as ECMA arrays.
    EcmaArray,
    /// LocalConnection uses strict arrays only when every array index exists.
    StrictWhenDense,
    /// NetConnection uses strict arrays whenever there are no custom properties.
    StrictWhenNoCustomProperties,
}

/// The wire representation and effective length selected for an AVM1 array.
struct ArraySerializationInfo {
    is_strict_array: bool,
    length: usize,
}

/// Select an array representation using the rules of its transport.
///
/// Keep this shared by both AMF-value and LSO-writer serialization so nested
/// and top-level arrays apply exactly the same classification and length rules.
fn array_serialization_info(
    keys: &[AvmString],
    mut length: usize,
    array_encoding: ArrayEncoding,
) -> ArraySerializationInfo {
    let is_strict_array = match array_encoding {
        ArrayEncoding::EcmaArray => false,
        ArrayEncoding::StrictWhenDense => {
            for key in keys {
                if let Some(index) = parse_canonical_array_index(key.as_wstr()) {
                    length = length.max(index as usize + 1);
                }
            }

            // LocalConnection preserves holes and noncanonical numeric-looking
            // property names, so only genuinely dense arrays use strict encoding.
            keys.len() == length
                && keys.iter().all(|key| {
                    parse_canonical_array_index(key.as_wstr())
                        .is_some_and(|index| (index as usize) < length)
                })
        }
        ArrayEncoding::StrictWhenNoCustomProperties => {
            let mut has_custom_properties = false;
            let mut strict_array_length = length;
            for key in keys {
                if let Some(index) = net_connection_array_index(key.to_utf8_lossy().as_ref()) {
                    if let Some(end) = index.checked_add(1) {
                        strict_array_length = strict_array_length.max(end as usize);
                    }
                } else {
                    has_custom_properties = true;
                }
            }

            if !has_custom_properties {
                length = strict_array_length;
            }

            // NetConnection treats sparse arrays with only numeric keys as
            // strict arrays, serializing holes as explicit undefined values.
            !has_custom_properties
        }
    };

    ArraySerializationInfo {
        is_strict_array,
        length,
    }
}

/// Helper to serialize a specific value into the ObjWriter cache architecture.
fn serialize_value_to_writer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &str,
    elem: Value<'gc>,
    writer: &mut dyn ObjWriter<'_>,
    array_encoding: ArrayEncoding,
) {
    match elem {
        Value::Object(o) => {
            if o.as_function().is_some() {
                // Flash entirely skips functions during object serialization.
            } else if o.as_display_object().is_some() {
                writer.undefined(name)
            } else if o.is_native_array() {
                let length = o.length(activation).unwrap_or(0).max(0) as usize;
                let keys = o.get_keys(activation, false);
                let array_info = array_serialization_info(&keys, length, array_encoding);

                if array_info.is_strict_array {
                    let (aw, token) = writer.strict_array(CacheKey::from_ptr(o.as_ptr()));

                    if let Some(mut aw) = aw {
                        for i in 0..array_info.length {
                            let prop_value = o.get_element(activation, i as i32);
                            serialize_value_to_writer(
                                activation,
                                "",
                                prop_value,
                                &mut aw,
                                array_encoding,
                            );
                        }

                        aw.commit(name);
                    } else {
                        writer.reference(name, token);
                    }
                } else {
                    let (aw, token) = writer.array(CacheKey::from_ptr(o.as_ptr()));

                    if let Some(mut aw) = aw {
                        recursive_serialize(activation, o, &mut aw, array_encoding);
                        aw.set_length(array_info.length as u32);
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
                if array_encoding == ArrayEncoding::EcmaArray {
                    // SharedObject LSO bodies do not include XML in their reference table.
                    writer.add_element(
                        name,
                        AmfValue::XML {
                            value: string.to_utf8_lossy().into_owned(),
                            is_string: true,
                        },
                    );
                } else {
                    // AMF remoting does include XML in the reference table.
                    writer.xml(
                        CacheKey::from_ptr(o.as_ptr()),
                        name,
                        string.to_utf8_lossy().as_ref(),
                        true,
                    );
                }
            } else if let NativeObject::Date(date) = o.native() {
                // Flash resolves the global Date constructor while serializing AMF0 dates.
                let global = activation.global_object();
                let _ = global.get(istr!("Date"), activation);

                writer.date(
                    CacheKey::from_ptr(o.as_ptr()),
                    name,
                    date.get().time(),
                    None,
                )
            } else {
                let key = CacheKey::from_ptr(o.as_ptr());

                if let Some(class_name) = object_class_name(activation, o) {
                    let (ow, token) = writer.typed_object(&class_name, key);
                    if let Some(mut ow) = ow {
                        recursive_serialize(activation, o, &mut ow, array_encoding);
                        ow.commit(name);
                    } else {
                        writer.reference(name, token);
                    }
                } else {
                    let (ow, token) = writer.object(key);
                    if let Some(mut ow) = ow {
                        recursive_serialize(activation, o, &mut ow, array_encoding);
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
pub(crate) fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: Object<'gc>,
    writer: &mut dyn ObjWriter<'_>,
    array_encoding: ArrayEncoding,
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
        serialize_value_to_writer(activation, name.as_ref(), elem, writer, array_encoding);
    }
}

/// Deserialize an AmfValue to a Value, using AMF remoting reference-table semantics.
pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
) -> Value<'gc> {
    let mut deferred_dates = Vec::new();
    deserialize_value_inner(
        activation,
        val,
        reference_cache,
        true,
        false,
        &mut deferred_dates,
    )
}

/// Deserialize an LSO value, where XML values do not occupy reference-table slots.
/// Used by SharedObject deserialization.
pub(crate) fn deserialize_lso_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
) -> Value<'gc> {
    let mut deferred_dates = Vec::new();
    deserialize_value_inner(
        activation,
        val,
        reference_cache,
        false,
        false,
        &mut deferred_dates,
    )
}

/// Deserialize a LocalConnection LSO value, applying LocalConnection-specific class behavior.
fn deserialize_local_connection_lso_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
    deferred_dates: &mut Vec<DeferredAmf0Date<'gc>>,
) -> Value<'gc> {
    deserialize_value_inner(
        activation,
        val,
        reference_cache,
        false,
        true,
        deferred_dates,
    )
}

fn deserialize_value_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    reference_cache: &mut Amf0ReferenceCache<'gc>,
    xml_is_referenceable: bool,
    local_connection: bool,
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

            let value: Value<'gc> = obj.into();

            // Register before deserializing children so references back to this array work.
            reference_cache.insert(value);

            for entry in &data.elements {
                let child = deserialize_value_inner(
                    activation,
                    entry.value(),
                    reference_cache,
                    xml_is_referenceable,
                    local_connection,
                    deferred_dates,
                );

                if let Some(i) = parse_canonical_array_index(from_utf8(entry.name()).as_ref()) {
                    obj.set_element(activation, i, child).unwrap();
                } else {
                    obj.define_value(
                        activation.gc(),
                        AvmString::new_utf8(activation.gc(), &entry.name),
                        child,
                        Attribute::empty(),
                    );
                }
            }

            value
        }
        AmfValue::StrictArray { values, .. } => {
            let obj = ArrayBuilder::empty_with_len(
                activation,
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            );

            let value: Value<'gc> = obj.into();

            // Register before deserializing children so self-references work.
            reference_cache.insert(value);

            for (i, item) in values.iter().enumerate() {
                let child = deserialize_value_inner(
                    activation,
                    item,
                    reference_cache,
                    xml_is_referenceable,
                    local_connection,
                    deferred_dates,
                );
                obj.set_element(activation, i as i32, child).unwrap();
            }

            value
        }
        AmfValue::Object { data, .. } => {
            // Deserialize Object
            let obj = Object::new(
                &activation.context.strings,
                Some(activation.prototypes().object),
            );

            let value: Value<'gc> = obj.into();

            // Register before deserializing children so self-references work.
            reference_cache.insert(value);

            for entry in &data.elements {
                let child = deserialize_value_inner(
                    activation,
                    entry.value(),
                    reference_cache,
                    xml_is_referenceable,
                    local_connection,
                    deferred_dates,
                );
                let name = AvmString::new_utf8(activation.gc(), &entry.name);
                obj.define_value(activation.gc(), name, child, Attribute::empty());
            }

            value
        }
        AmfValue::Date { data, .. } => {
            let value = if local_connection {
                local_date_from_amf0(activation, data.time, deferred_dates)
            } else {
                date_from_amf0(activation, data.time)
            };

            reference_cache.insert(value);
            value
        }
        AmfValue::XML { value: content, .. } => {
            // Flash creates a native XML document using the current XML prototype,
            // without invoking the script-visible XML constructor.
            let value = xml_from_amf0(activation, content);

            if xml_is_referenceable {
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

const MAX_AMF0_REFERENCES: usize = u16::MAX as usize + 1;

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

/// Create a LocalConnection reference table containing the top-level argument
/// and placeholders for objects in earlier properties. Those earlier objects
/// keep their wire indices, but Flash does not preserve their identity when a
/// later property refers to them.
fn local_connection_reference_cache<'gc>(
    root: Value<'gc>,
    references_before_property: usize,
) -> Amf0ReferenceCache<'gc> {
    let mut reference_cache = Amf0ReferenceCache::default();
    reference_cache.insert(root);
    for _ in 1..references_before_property.min(MAX_AMF0_REFERENCES) {
        reference_cache.insert(Value::Undefined);
    }
    reference_cache
}

/// Deserialize a LocalConnection argument. Reference indices are still counted
/// across the argument, but values from earlier sibling properties are not
/// carried into a later property's cache.
pub(crate) fn deserialize_local_connection_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    val: &AmfValue,
    deferred_dates: &mut Vec<DeferredAmf0Date<'gc>>,
) -> Value<'gc> {
    match val {
        AmfValue::Object { data, .. } => {
            let obj = Object::new(
                &activation.context.strings,
                Some(activation.prototypes().object),
            );
            let root: Value<'gc> = obj.into();
            let mut references_before_property = 1;

            for entry in &data.elements {
                let mut reference_cache =
                    local_connection_reference_cache(root, references_before_property);
                let child = deserialize_local_connection_lso_value(
                    activation,
                    entry.value(),
                    &mut reference_cache,
                    deferred_dates,
                );
                let name = AvmString::new_utf8(activation.gc(), &entry.name);
                obj.define_value(activation.gc(), name, child, Attribute::empty());
                references_before_property = references_before_property
                    .saturating_add(count_amf0_references(entry.value()))
                    .min(MAX_AMF0_REFERENCES);
            }

            root
        }
        AmfValue::ECMAArray { data, .. } => {
            let Ok(Some(Value::Object(obj))) =
                activation.instantiate_class_fast([istr!("Array")], &[Value::from(data.length)])
            else {
                return Value::Undefined;
            };
            let root: Value<'gc> = obj.into();
            let mut references_before_property = 1;

            for (index, item) in data.dense.iter().enumerate() {
                let mut reference_cache =
                    local_connection_reference_cache(root, references_before_property);
                let child = deserialize_local_connection_lso_value(
                    activation,
                    item,
                    &mut reference_cache,
                    deferred_dates,
                );
                obj.set_element(activation, index as i32, child).unwrap();
                references_before_property = references_before_property
                    .saturating_add(count_amf0_references(item))
                    .min(MAX_AMF0_REFERENCES);
            }

            for entry in &data.elements {
                let mut reference_cache =
                    local_connection_reference_cache(root, references_before_property);
                let child = deserialize_local_connection_lso_value(
                    activation,
                    entry.value(),
                    &mut reference_cache,
                    deferred_dates,
                );

                if let Some(index) = parse_canonical_array_index(from_utf8(entry.name()).as_ref()) {
                    obj.set_element(activation, index, child)
                        .expect("Failed to set a sparse LocalConnection array element");
                } else {
                    obj.define_value(
                        activation.gc(),
                        AvmString::new_utf8(activation.gc(), &entry.name),
                        child,
                        Attribute::empty(),
                    );
                }

                references_before_property = references_before_property
                    .saturating_add(count_amf0_references(entry.value()))
                    .min(MAX_AMF0_REFERENCES);
            }

            root
        }
        AmfValue::StrictArray { values, .. } => {
            let args = &[Value::from_usize_lossy(values.len())];
            let Ok(Some(Value::Object(obj))) =
                activation.instantiate_class_fast([istr!("Array")], args)
            else {
                return Value::Undefined;
            };
            let root: Value<'gc> = obj.into();
            let mut references_before_property = 1;

            for (index, item) in values.iter().enumerate() {
                let mut reference_cache =
                    local_connection_reference_cache(root, references_before_property);
                let child = deserialize_local_connection_lso_value(
                    activation,
                    item,
                    &mut reference_cache,
                    deferred_dates,
                );
                obj.set_element(activation, index as i32, child)
                    .expect("Failed to set a dense LocalConnection array element");
                references_before_property = references_before_property
                    .saturating_add(count_amf0_references(item))
                    .min(MAX_AMF0_REFERENCES);
            }

            root
        }
        _ => {
            let mut reference_cache = Amf0ReferenceCache::default();
            deserialize_local_connection_lso_value(
                activation,
                val,
                &mut reference_cache,
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
    fn array_serialization_info_applies_transport_specific_rules() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let dense_keys = vec![AvmString::new_utf8(activation.gc(), "0")];
            let dense = array_serialization_info(&dense_keys, 0, ArrayEncoding::StrictWhenDense);
            assert!(dense.is_strict_array);
            assert_eq!(dense.length, 1);

            let sparse_keys = vec![
                AvmString::new_utf8(activation.gc(), "0"),
                AvmString::new_utf8(activation.gc(), "3"),
            ];
            let local_connection =
                array_serialization_info(&sparse_keys, 4, ArrayEncoding::StrictWhenDense);
            assert!(!local_connection.is_strict_array);
            assert_eq!(local_connection.length, 4);

            let net_connection = array_serialization_info(
                &sparse_keys,
                4,
                ArrayEncoding::StrictWhenNoCustomProperties,
            );
            assert!(net_connection.is_strict_array);
            assert_eq!(net_connection.length, 4);

            let custom_keys = vec![AvmString::new_utf8(activation.gc(), "+1")];
            let net_connection_custom = array_serialization_info(
                &custom_keys,
                0,
                ArrayEncoding::StrictWhenNoCustomProperties,
            );
            assert!(!net_connection_custom.is_strict_array);

            let mixed_keys = vec![
                AvmString::new_utf8(activation.gc(), "2147483648"),
                AvmString::new_utf8(activation.gc(), "4294967296"),
            ];
            let net_connection_mixed = array_serialization_info(
                &mixed_keys,
                1,
                ArrayEncoding::StrictWhenNoCustomProperties,
            );
            assert!(!net_connection_mixed.is_strict_array);
            assert_eq!(net_connection_mixed.length, 1);

            let shared_object = array_serialization_info(&dense_keys, 1, ArrayEncoding::EcmaArray);
            assert!(!shared_object.is_strict_array);

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
            let Value::Object(array) =
                deserialize_local_connection_value(activation, &value, &mut deferred_dates)
            else {
                panic!("LocalConnection ECMA array should deserialize to an array");
            };

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
