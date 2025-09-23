use crate::avm2::array::ArrayStorage;
use crate::avm2::object::{ArrayObject, Object, ScriptObject, TObject};
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

use gc_arena::Collect;
use ruffle_macros::istr;
use swf::avm2::types::{Index as AbcIndex, Metadata as AbcMetadata};

// Represents a single key-value pair for a trait metadata.
#[derive(Clone, Collect, Debug, Eq, PartialEq)]
#[collect(no_drop)]
struct MetadataItem<'gc> {
    key: AvmString<'gc>,
    value: AvmString<'gc>,
}

// Represents a single metadata item for a trait.
#[derive(Clone, Collect, Debug, Eq, PartialEq)]
#[collect(no_drop)]
pub struct Metadata<'gc> {
    name: AvmString<'gc>,
    items: Vec<MetadataItem<'gc>>,
}

impl<'gc> Metadata<'gc> {
    // Converts an AbcMetadata into a Metadata by resolving all the indexes.
    pub fn from_abc_index(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        metadata: &[AbcIndex<AbcMetadata>],
    ) -> Result<Option<Box<[Metadata<'gc>]>>, Error<'gc>> {
        if metadata.is_empty() {
            return Ok(None);
        }

        let abc = translation_unit.abc();
        let mut trait_metadata_list = vec![];
        for single_metadata in metadata.iter() {
            // Lookup the Index<Metadata> to convert it into a Metadata.
            let single_metadata = abc
                .metadata
                .get(single_metadata.0 as usize)
                .ok_or_else(|| format!("Unknown metadata {}", single_metadata.0))?;

            let name =
                translation_unit.pool_string(single_metadata.name.0, activation.strings())?;

            let mut current_metadata_items = vec![];
            for metadata_item in single_metadata.items.iter() {
                let key =
                    translation_unit.pool_string(metadata_item.key.0, activation.strings())?;

                let value =
                    translation_unit.pool_string(metadata_item.value.0, activation.strings())?;

                let item = MetadataItem {
                    key: key.into(),
                    value: value.into(),
                };
                current_metadata_items.push(item);
            }

            let single_metadata_result = Metadata {
                name: name.into(),
                items: current_metadata_items,
            };

            trait_metadata_list.push(single_metadata_result);
        }

        Ok(Some(trait_metadata_list.into_boxed_slice()))
    }

    // Converts the Metadata to an Object of the form used in avmplus:describeTypeJSON().
    pub fn as_json_object(&self, activation: &mut Activation<'_, 'gc>) -> Object<'gc> {
        let object = ScriptObject::new_object(activation);
        object.set_dynamic_property(istr!("name"), self.name.into(), activation.gc());

        let values = self
            .items
            .iter()
            .map(|item| {
                let value_object = ScriptObject::new_object(activation);
                value_object.set_dynamic_property(istr!("key"), item.key.into(), activation.gc());
                value_object.set_dynamic_property(
                    istr!("value"),
                    item.value.into(),
                    activation.gc(),
                );
                Some(value_object.into())
            })
            .collect::<Vec<Option<Value<'gc>>>>();

        let values_array =
            ArrayObject::from_storage(activation, ArrayStorage::from_storage(values));
        object.set_dynamic_property(istr!("value"), values_array.into(), activation.gc());
        object
    }
}
