use super::{Object, string::AvmString};
use fnv::FnvBuildHasher;
use gc_arena::Collect;
use gc_arena::collect::Trace;
use hashbrown::HashTable;
use std::cell::Cell;
use std::fmt::{self, Debug};
use std::hash::{BuildHasher, Hash};

#[derive(Debug, Collect, Copy, Clone)]
#[collect(no_drop)]
pub struct DynamicProperty<V> {
    pub value: V,
    pub enumerable: bool,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum DynamicKey<'gc> {
    String(AvmString<'gc>),
    // When the name parses as a non-negative integer, we use that integer as the key
    // See `ScriptObject::get_property_local` and `ScriptObject::set_property_local`.
    // This is observable when iterating over the object keys, as the key
    // can be `number`
    Uint(u32),
    Object(Object<'gc>),
}

/// A HashMap designed for dynamic properties on an object.
///
/// Uses `HashTable` directly to expose stable bucket indices, which are
/// needed for correct iteration when entries are added or removed mid-iteration.
#[derive(Clone)]
pub struct DynamicMap<K, V> {
    table: HashTable<(K, DynamicProperty<V>)>,
    hasher: FnvBuildHasher,
    public_index: Cell<usize>,
    // The actual bucket index that represents where an item is in the table
    real_index: Cell<usize>,
}

unsafe impl<'gc, K: Collect<'gc>, V: Collect<'gc>> Collect<'gc> for DynamicMap<K, V> {
    const NEEDS_TRACE: bool = K::NEEDS_TRACE || V::NEEDS_TRACE;

    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        match (K::NEEDS_TRACE, V::NEEDS_TRACE) {
            (true, true) => self.table.iter().for_each(|(k, v)| {
                cc.trace(k);
                cc.trace(&v.value);
            }),
            (true, false) => self.table.iter().for_each(|(k, _)| cc.trace(k)),
            (false, true) => self.table.iter().for_each(|(_, v)| cc.trace(&v.value)),
            (false, false) => {}
        }
    }
}

impl<K: Debug, V: Debug> Debug for DynamicMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicMap")
            .field("table", &self.table)
            .field("public_index", &self.public_index)
            .field("real_index", &self.real_index)
            .finish()
    }
}

impl<K: Eq + Hash, V> Default for DynamicMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V> DynamicMap<K, V> {
    fn hash_key(&self, key: &K) -> u64 {
        self.hasher.hash_one(key)
    }

    pub fn new() -> Self {
        Self {
            table: HashTable::new(),
            hasher: FnvBuildHasher::default(),
            public_index: Cell::new(0),
            real_index: Cell::new(0),
        }
    }

    pub fn get(&self, key: &K) -> Option<&DynamicProperty<V>> {
        let hash = self.hash_key(key);

        self.table.find(hash, |(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let hash = self.hash_key(key);

        self.table.find(hash, |(k, _)| k == key).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &DynamicProperty<V>)> {
        self.table.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.table.iter().map(|(k, _)| k)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Gets the bucket index from the current public index.
    /// Returns `None` if the index is out of bounds.
    fn public_to_real_index(&self, index: usize) -> Option<usize> {
        let mut count = 0;
        let num_buckets = self.table.num_buckets();

        for i in 0..num_buckets {
            if let Some((_, v)) = self.table.get_bucket(i) {
                if v.enumerable {
                    count += 1;

                    if count >= index {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    pub fn insert(&mut self, key: K, new_value: V) {
        let hash = self.hash_key(&key);

        match self.table.find_mut(hash, |(k, _)| *k == key) {
            Some((_, prop)) => {
                // NOTE: When inserting a new value into an already-occupied entry,
                // the value of the `enumerable` field isn't reset to `true`
                prop.value = new_value;
            }
            None => {
                self.table.insert_unique(
                    hash,
                    (
                        key,
                        DynamicProperty {
                            value: new_value,
                            enumerable: true,
                        },
                    ),
                    |(k, _)| self.hasher.hash_one(k),
                );
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<DynamicProperty<V>> {
        let hash = self.hash_key(key);

        match self.table.find_entry(hash, |(k, _)| k == key) {
            Ok(occupied) => Some(occupied.remove().0.1),
            Err(_) => None,
        }
    }

    pub fn next(&self, index: usize) -> Option<usize> {
        // Start iteration from the beginning
        if index == 0 {
            if let Some(real) = self.public_to_real_index(1) {
                self.real_index.set(real);
                self.public_index.set(1);
                return Some(1);
            } else {
                self.public_index.set(0);
                self.real_index.set(0);
                return None;
            }
        }
        // If the public index is zero than this is the first time this is being enumerated,
        // if index != self.public_index, then we are enumerating twice OR we mutated while enumerating.
        //
        // Regardless of the reason, we just need to sync the supplied index to the real index.
        if self.public_index.get() == 0 || index != self.public_index.get() {
            if let Some(real) = self.public_to_real_index(index) {
                // Pick up where we left off in the iteration
                // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/avmplusHashtable.cpp#L395
                self.real_index.set(real);
                self.public_index.set(index);
            } else {
                self.public_index.set(0);
                self.real_index.set(0);
                return None;
            }
        }

        let real = self.real_index.get() + 1;
        let num_buckets = self.table.num_buckets();

        if !self.table.is_empty() && real < num_buckets {
            for i in real..num_buckets {
                if let Some((_, v)) = self.table.get_bucket(i) {
                    if v.enumerable {
                        self.real_index.set(i);
                        self.public_index.set(self.public_index.get() + 1);
                        return Some(self.public_index.get());
                    }
                }
            }
        }

        None
    }

    pub fn pair_at(&self, index: usize) -> Option<&(K, DynamicProperty<V>)> {
        let real_index = if self.public_index.get() == 0 || self.public_index.get() != index {
            self.public_to_real_index(index)?
        } else {
            self.real_index.get()
        };

        self.table.get_bucket(real_index)
    }

    pub fn key_at(&self, index: usize) -> Option<&K> {
        self.pair_at(index).map(|(k, _)| k)
    }

    pub fn value_at(&self, index: usize) -> Option<&V> {
        self.pair_at(index).map(|(_, p)| &p.value)
    }

    pub fn set_enumerable(&mut self, key: &K, enumerable: bool) {
        let hash = self.hash_key(key);

        if let Some((_, prop)) = self.table.find_mut(hash, |(k, _)| k == key) {
            prop.enumerable = enumerable;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::DynamicMap;

    #[test]
    fn test_dynamic_map() {
        let mut map: DynamicMap<&'static str, i32> = DynamicMap::new();
        map.insert("a", 0);
        map.insert("b", 0);
        map.insert("c", 0);
        map.insert("d", 0);
        let mut current = 0;
        while let Some(next) = map.next(current) {
            if current == 2 {
                map.insert("e", 0);
                map.insert("f", 0);
            }
            println!("{}", map.key_at(current).unwrap());
            current = next;
        }
        println!("next");
        current = 0;
        while let Some(next) = map.next(current) {
            println!("{}", map.key_at(current).unwrap());
            current = next;
        }
    }
}
