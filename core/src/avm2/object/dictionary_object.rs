//! Object representation for `flash.utils.Dictionary`

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::dynamic_map::DynamicKey;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::value::Value;
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::RefLock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use std::cell::Cell;

/// A class instance allocator that allocates Dictionary objects.
pub fn dictionary_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(DictionaryObject::new(class, false, activation).into())
}

impl<'gc> DictionaryObject<'gc> {
    /// Allocate a fresh Dictionary instance.
    /// Used both by the default allocator (weak_keys=false) and by the
    /// `[Ruffle(CustomConstructor)]` handler (weak_keys reflecting the
    /// `weakKeys` argument).
    pub fn new(
        class: ClassObject<'gc>,
        weak_keys: bool,
        activation: &mut Activation<'_, 'gc>,
    ) -> Self {
        let base = ScriptObjectData::new(class);
        DictionaryObject(Gc::new(
            activation.gc(),
            DictionaryObjectData {
                base,
                weak_keys: Cell::new(weak_keys),
                weak_entries: RefLock::new(Vec::new()),
            },
        ))
    }
}

/// An object that allows associations between objects and values.
///
/// This is implemented by way of "object space", parallel to the property
/// space that ordinary properties live in. This space has no namespaces, and
/// keys are objects instead of strings.
///
/// When constructed with `new Dictionary(true)`, object keys are held weakly:
/// entries are transparently dropped when the only references to the key
/// are the ones inside this dictionary. This matches Flash's documented
/// behavior and is required by framework code that maintains object→value
/// caches (renderer-per-item mappings, watcher-per-source registries) and
/// must not extend the lifetime of the key objects.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DictionaryObject<'gc>(pub Gc<'gc, DictionaryObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DictionaryObjectWeak<'gc>(pub GcWeak<'gc, DictionaryObjectData<'gc>>);

impl fmt::Debug for DictionaryObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct DictionaryObjectData<'gc> {
    /// Base script object. When weak_keys is false, object keys live here.
    base: ScriptObjectData<'gc>,

    /// `true` when constructed with `weakKeys=true`. Set once at
    /// construction by `DictionaryObject::new` from the AS3 constructor
    /// argument; immutable afterwards (matches Flash semantics).
    #[collect(require_static)]
    weak_keys: Cell<bool>,

    /// Object-keyed entries when `weak_keys` is true. A linear `Vec` rather
    /// than a hashtable: weak pointers become unhashable when their target
    /// is collected, and typical weak-keyed Dictionary use is a small cache
    /// (tens of entries), so linear scan beats maintaining a hashtable of
    /// `GcWeak` keys.
    weak_entries: RefLock<Vec<WeakEntry<'gc>>>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
struct WeakEntry<'gc> {
    /// Weak reference to the key object. When `upgrade(mc)` returns `None`,
    /// the entry is considered dead and is dropped at the next mutating
    /// access on the dictionary.
    key: crate::avm2::object::WeakObject<'gc>,
    value: Value<'gc>,
}

impl<'gc> DictionaryObject<'gc> {
    /// Retrieve a value in the dictionary's object space.
    pub fn get_property_by_object(self, name: Object<'gc>) -> Value<'gc> {
        if self.0.weak_keys.get() {
            let entries = self.0.weak_entries.borrow();
            for e in entries.iter() {
                // Pointer identity is enough: as_ptr() is stable for the
                // allocation, and matches the strong-key behavior.
                if std::ptr::eq(e.key.as_ptr(), name.as_ptr()) {
                    return e.value;
                }
            }
            return Value::Undefined;
        }
        self.base()
            .values()
            .get(&DynamicKey::Object(name))
            .map(|v| v.value)
            .unwrap_or(Value::Undefined)
    }

    /// Set a value in the dictionary's object space.
    ///
    /// For `weakKeys=true`, this also drops entries whose keys are no
    /// longer reachable, combining the drop with the pointer-equality
    /// scan that is already needed to detect an existing entry to
    /// replace (single pass). Without this drop, the values associated
    /// with collected keys would be retained indefinitely; code that
    /// populates a weak-keyed Dictionary as a write-mostly cache (only
    /// insertions, no iteration) accumulates them unbounded. Adobe
    /// Flash Player drops dead-key entries via its weak GC; `gc-arena`
    /// does not, so the explicit drop here is required to match the
    /// documented Flash semantics.
    pub fn set_property_by_object(self, name: Object<'gc>, value: Value<'gc>, mc: &Mutation<'gc>) {
        if self.0.weak_keys.get() {
            let mut entries =
                unlock!(Gc::write(mc, self.0), DictionaryObjectData, weak_entries).borrow_mut();
            // Single pass: drop dead-key entries while looking for an existing
            // entry to replace. Both operations were already O(N) in isolation;
            // combining costs no extra walks.
            let mut found = false;
            entries.retain_mut(|e| {
                if e.key.upgrade(mc).is_none() {
                    return false; // dead key — drop entry, freeing value
                }
                if !found && std::ptr::eq(e.key.as_ptr(), name.as_ptr()) {
                    e.value = value;
                    found = true;
                }
                true
            });
            if !found {
                entries.push(WeakEntry {
                    key: name.downgrade(),
                    value,
                });
            }
            return;
        }
        self.base()
            .values_mut(mc)
            .insert(DynamicKey::Object(name), value);
    }

    /// Delete a value from the dictionary's object space.
    pub fn delete_property_by_object(self, name: Object<'gc>, mc: &Mutation<'gc>) {
        if self.0.weak_keys.get() {
            let mut entries =
                unlock!(Gc::write(mc, self.0), DictionaryObjectData, weak_entries).borrow_mut();
            entries.retain(|e| !std::ptr::eq(e.key.as_ptr(), name.as_ptr()));
            return;
        }
        self.base().values_mut(mc).remove(&DynamicKey::Object(name));
    }

    pub fn has_property_by_object(self, name: Object<'gc>) -> bool {
        if self.0.weak_keys.get() {
            let entries = self.0.weak_entries.borrow();
            for e in entries.iter() {
                if std::ptr::eq(e.key.as_ptr(), name.as_ptr()) {
                    return true;
                }
            }
            return false;
        }
        self.base().values().contains_key(&DynamicKey::Object(name))
    }
}

impl<'gc> TObject<'gc> for DictionaryObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    // Calling `setPropertyIsEnumerable` on a `Dictionary` has no effect -
    // stringified properties are always enumerable.
    fn set_local_property_is_enumerable(
        &self,
        _mc: &Mutation<'gc>,
        _name: AvmString<'gc>,
        _is_enumerable: bool,
    ) {
    }

    fn get_next_enumerant(
        self,
        last_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<u32, Error<'gc>> {
        if !self.0.weak_keys.get() {
            return Ok(self.base().get_next_enumerant(last_index));
        }
        // Weak path: a weak-keyed Dictionary still admits non-object keys
        // (literal strings/numbers), which are stored in the base values map
        // — only object keys go in `weak_entries`. Enumeration walks the
        // base first, then `weak_entries`. The two phases are distinguished
        // by the index range: indices in `1..=base_count` address the base
        // (returned verbatim by `base.get_next_enumerant`); indices in
        // `base_count+1..=base_count+weak_entries.len()` address weak
        // entries (1-based position within the vec = `index - base_count`).
        let base_count = self.base().values().len() as u32;
        if last_index <= base_count {
            let next_in_base = self.base().get_next_enumerant(last_index);
            if next_in_base != 0 {
                return Ok(next_in_base);
            }
        }
        let weak_start = last_index.saturating_sub(base_count) as usize;
        let entries = self.0.weak_entries.borrow();
        let mc = activation.gc();
        for i in weak_start..entries.len() {
            if entries[i].key.upgrade(mc).is_some() {
                return Ok(base_count + (i as u32) + 1);
            }
        }
        Ok(0)
    }

    fn get_enumerant_name(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if !self.0.weak_keys.get() {
            return Ok(self.base().get_enumerant_name(index).unwrap_or(Value::Null));
        }
        let base_count = self.base().values().len() as u32;
        if index <= base_count {
            return Ok(self.base().get_enumerant_name(index).unwrap_or(Value::Null));
        }
        let weak_idx = (index - base_count - 1) as usize;
        let entries = self.0.weak_entries.borrow();
        let mc = activation.gc();
        if let Some(e) = entries.get(weak_idx)
            && let Some(obj) = e.key.upgrade(mc)
        {
            return Ok(Value::Object(obj));
        }
        Ok(Value::Null)
    }

    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if !self.0.weak_keys.get() {
            return Ok(*self
                .base()
                .values()
                .value_at(index as usize)
                .unwrap_or(&Value::Undefined));
        }
        let base_count = self.base().values().len() as u32;
        if index <= base_count {
            return Ok(*self
                .base()
                .values()
                .value_at(index as usize)
                .unwrap_or(&Value::Undefined));
        }
        let weak_idx = (index - base_count - 1) as usize;
        let entries = self.0.weak_entries.borrow();
        let mc = activation.gc();
        if let Some(e) = entries.get(weak_idx)
            && e.key.upgrade(mc).is_some()
        {
            return Ok(e.value);
        }
        Ok(Value::Undefined)
    }
}
