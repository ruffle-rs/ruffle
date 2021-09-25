//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;

/// A class instance allocator that allocates Dictionary objects.
pub fn dictionary_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(DictionaryObject(GcCell::allocate(
        activation.context.gc_context,
        DictionaryObjectData {
            base,
            object_space: HashMap::new(),
        },
    ))
    .into())
}

/// An object that allows associations between objects and values.
///
/// This is implemented by way of "object space", parallel to the property
/// space that ordinary properties live in. This space has no namespaces, and
/// keys are objects instead of strings.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct DictionaryObject<'gc>(GcCell<'gc, DictionaryObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct DictionaryObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Object key storage
    object_space: HashMap<Object<'gc>, Value<'gc>>,
}

impl<'gc> DictionaryObject<'gc> {
    /// Retrieve a value in the dictionary's object space.
    pub fn get_property_by_object(self, name: Object<'gc>) -> Value<'gc> {
        self.0
            .read()
            .object_space
            .get(&name)
            .cloned()
            .unwrap_or(Value::Undefined)
    }

    /// Set a value in the dictionary's object space.
    pub fn set_property_by_object(
        self,
        name: Object<'gc>,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) {
        self.0.write(mc).object_space.insert(name, value);
    }

    /// Delete a value from the dictionary's object space.
    pub fn delete_property_by_object(self, name: Object<'gc>, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).object_space.remove(&name);
    }

    pub fn has_property_by_object(self, name: Object<'gc>) -> bool {
        self.0.read().object_space.get(&name).is_some()
    }
}

impl<'gc> TObject<'gc> for DictionaryObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Object::from(*self).into())
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(DictionaryObject(GcCell::allocate(
            activation.context.gc_context,
            DictionaryObjectData {
                base,
                object_space: HashMap::new(),
            },
        ))
        .into())
    }

    fn as_dictionary_object(self) -> Option<DictionaryObject<'gc>> {
        Some(self)
    }

    fn get_enumerant_name(&self, index: u32) -> Option<Value<'gc>> {
        let read = self.0.read();
        let last_enumerant = read.base.get_last_enumerant();

        if index < last_enumerant {
            read.base.get_enumerant_name(index)
        } else {
            let object_space_index = index.saturating_sub(last_enumerant);

            read.object_space
                .keys()
                .nth(object_space_index as usize)
                .cloned()
                .map(|v| v.into())
        }
    }
}
