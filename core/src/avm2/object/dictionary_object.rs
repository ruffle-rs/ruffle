//! Object representation for `flash.utils.Dictionary`

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject, WeakObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use core::fmt;
use fnv::FnvBuildHasher;
use gc_arena::{Collect, GcCell, GcWeakCell, MutationContext};
use hashbrown::HashMap;
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Dictionary objects.
pub fn dictionary_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(DictionaryObject(GcCell::allocate(
        activation.context.gc_context,
        DictionaryObjectData {
            base,
            object_space: DictionaryMap::Strong(Default::default()),
        },
    ))
    .into())
}

/// An object that allows associations between objects and values.
///
/// This is implemented by way of "object space", parallel to the property
/// space that ordinary properties live in. This space has no namespaces, and
/// keys are objects instead of strings.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DictionaryObject<'gc>(pub GcCell<'gc, DictionaryObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DictionaryObjectWeak<'gc>(pub GcWeakCell<'gc, DictionaryObjectData<'gc>>);

impl fmt::Debug for DictionaryObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone)]
pub enum DictionaryMap<'gc> {
    Strong(HashMap<Object<'gc>, Value<'gc>, FnvBuildHasher>),
    Weak(HashMap<WeakObject<'gc>, Value<'gc>, FnvBuildHasher>),
}

unsafe impl<'gc> Collect for DictionaryMap<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Self::Strong(m) => {
                for (k, v) in m {
                    k.trace(cc);
                    v.trace(cc);
                }
            }
            Self::Weak(m) => {
                for (k, v) in m {
                    k.trace(cc);
                    v.trace(cc);
                }
            }
        }
    }
}

impl<'gc> DictionaryMap<'gc> {
    /// Inserts a value into this Dictionary.
    pub fn insert(&mut self, obj: Object<'gc>, v: Value<'gc>) {
        match self {
            DictionaryMap::Strong(m) => m.insert(obj, v),
            DictionaryMap::Weak(m) => m.insert(obj.downgrade(), v),
        };
    }

    /// Removes a value from this Dictionary.
    pub fn remove(&mut self, obj: Object<'gc>) {
        match self {
            DictionaryMap::Strong(m) => m.remove(&obj),
            DictionaryMap::Weak(m) => m.remove(&obj.downgrade()),
        };
    }

    /// Gets a value in this Dictionary by key.
    pub fn get(&self, obj: Object<'gc>) -> Option<Value<'gc>> {
        match self {
            DictionaryMap::Strong(m) => m.get(&obj).cloned(),
            DictionaryMap::Weak(m) => m.get(&obj.downgrade()).cloned(),
        }
    }

    /// Gets a key at a specific index in this Dictionary
    ///
    /// Automatically clears dead references
    pub fn get_key_at(
        &mut self,
        mc: MutationContext<'gc, '_>,
        index: usize,
    ) -> Option<Object<'gc>> {
        self.prune(mc);
        match self {
            DictionaryMap::Strong(m) => m.keys().nth(index).cloned(),
            DictionaryMap::Weak(m) => m.keys().nth(index).and_then(|o| o.upgrade(mc)),
        }
    }

    /// Clears out any dead references in this Dictionary
    pub fn prune(&mut self, mc: MutationContext<'gc, '_>) {
        if let Self::Weak(m) = self {
            m.drain_filter(|k, _| k.upgrade(mc).is_none());
        }
    }

    pub fn len(&self) -> usize {
        match self {
            DictionaryMap::Strong(m) => m.len(),
            DictionaryMap::Weak(m) => m.len(),
        }
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct DictionaryObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Object key storage
    object_space: DictionaryMap<'gc>,
}

impl<'gc> DictionaryObject<'gc> {
    /// Retrieve a value in the dictionary's object space.
    pub fn get_property_by_object(self, name: Object<'gc>) -> Value<'gc> {
        self.0
            .read()
            .object_space
            .get(name)
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
        self.0.write(mc).object_space.remove(name);
    }

    pub fn has_property_by_object(self, name: Object<'gc>) -> bool {
        self.0.read().object_space.get(name).is_some()
    }

    pub fn make_weak(self, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).object_space = DictionaryMap::Weak(Default::default());
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

    fn downgrade(&self) -> WeakObject<'gc> {
        WeakObject::DictionaryObject(DictionaryObjectWeak(GcCell::downgrade(self.0)))
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Object::from(*self).into())
    }

    fn as_dictionary_object(self) -> Option<DictionaryObject<'gc>> {
        Some(self)
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<u32>, Error<'gc>> {
        let mut write = self.0.write(activation.context.gc_context);
        let num_enumerants = write.base.num_enumerants();
        write.object_space.prune(activation.context.gc_context);
        let object_space_length = write.object_space.len() as u32;

        if last_index < num_enumerants + object_space_length {
            Ok(Some(last_index.saturating_add(1)))
        } else {
            Ok(None)
        }
    }

    fn get_enumerant_name(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let mut write = self.0.write(activation.context.gc_context);
        let object_space_len = write.object_space.len() as u32;
        if object_space_len >= index {
            Ok(index
                .checked_sub(1)
                .and_then(|index| {
                    write
                        .object_space
                        .get_key_at(activation.context.gc_context, index as usize)
                })
                .map(|v| v.into())
                .unwrap_or(Value::Undefined))
        } else {
            Ok(write
                .base
                .get_enumerant_name(index - object_space_len)
                .unwrap_or(Value::Undefined))
        }
    }

    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let name_value = self.get_enumerant_name(index, activation)?;
        if !name_value.is_primitive() {
            Ok(self.get_property_by_object(name_value.as_object().unwrap()))
        } else {
            self.get_public_property(name_value.coerce_to_string(activation)?, activation)
        }
    }

    // Calling `setPropertyIsEnumerable` on a `Dictionary` has no effect -
    // stringified properties are always enumerable.
    fn set_local_property_is_enumerable(
        &self,
        _mc: MutationContext<'gc, '_>,
        _name: AvmString<'gc>,
        _is_enumerable: bool,
    ) {
    }
}
