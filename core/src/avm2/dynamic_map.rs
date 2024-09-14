use fnv::FnvBuildHasher;
use gc_arena::Collect;
use hashbrown::raw::RawTable;
use std::{cell::Cell, hash::Hash};

use super::{string::AvmString, Object};

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
#[derive(Debug, Collect, Clone)]
#[collect(no_drop)]
pub struct DynamicMap<K: Eq + PartialEq + Hash, V> {
    values: hashbrown::HashMap<K, DynamicProperty<V>, FnvBuildHasher>,
    // The last index that was given back to flash
    public_index: Cell<usize>,
    // The actual index that represents where an item is in the HashMap
    real_index: Cell<usize>,
}

impl<K: Eq + PartialEq + Hash, V> Default for DynamicMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + PartialEq + Hash, V> DynamicMap<K, V> {
    pub fn new() -> Self {
        Self {
            values: hashbrown::HashMap::default(),
            public_index: Cell::new(0),
            real_index: Cell::new(0),
        }
    }

    pub fn as_hashmap(&self) -> &hashbrown::HashMap<K, DynamicProperty<V>, FnvBuildHasher> {
        &self.values
    }

    pub fn entry(
        &mut self,
        key: K,
    ) -> hashbrown::hash_map::Entry<K, DynamicProperty<V>, FnvBuildHasher> {
        self.values.entry(key)
    }

    /// Gets the real index from the current public index, returns false if real index is out of bounds
    fn public_to_real_index(&self, index: usize) -> Option<usize> {
        let mut count = 0;
        let raw = self.raw();
        if raw.is_empty() {
            return None;
        }
        for i in 0..raw.buckets() {
            unsafe {
                // SAFETY: It is impossible for i to be greater than the total buckets.
                if raw.is_bucket_full(i) {
                    // SAFETY: We know that this bucket is safe to access because we just checked
                    // that it is full.
                    let bucket = raw.bucket(i).as_ref();
                    if bucket.1.enumerable {
                        count += 1;
                        if count >= index {
                            return Some(i);
                        }
                    }
                }
            }
        }
        None
    }

    fn raw(&self) -> &RawTable<(K, DynamicProperty<V>)> {
        self.values.raw_table()
    }

    pub fn remove(&mut self, key: &K) -> Option<DynamicProperty<V>> {
        self.values.remove(key)
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
        let raw = self.raw();
        let total_buckets = raw.buckets();
        if !raw.is_empty() && real < total_buckets {
            for i in real..total_buckets {
                unsafe {
                    // SAFETY: It is impossible for i to be greater than the total buckets.
                    if raw.is_bucket_full(i) {
                        // SAFETY: We know that this bucket is safe to access because we just checked
                        // that it is full.
                        let bucket = raw.bucket(i).as_ref();
                        if bucket.1.enumerable {
                            self.real_index.set(i);
                            self.public_index.set(self.public_index.get() + 1);
                            return Some(self.public_index.get());
                        }
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
        if !self.values.is_empty() && real_index < self.raw().buckets() {
            unsafe {
                let bucket = self.raw().bucket(real_index);
                return Some(bucket.as_ref());
            }
        }
        None
    }

    pub fn key_at(&self, index: usize) -> Option<&K> {
        self.pair_at(index).map(|p| &p.0)
    }

    pub fn value_at(&self, index: usize) -> Option<&V> {
        self.pair_at(index).map(|p| &p.1.value)
    }
}

impl<K, V> DynamicMap<K, V>
where
    K: Eq + Hash,
{
    pub fn insert(&mut self, key: K, value: V) {
        self.values.insert(
            key,
            DynamicProperty {
                value,
                enumerable: true,
            },
        );
    }
    pub fn insert_no_enum(&mut self, key: K, value: V) {
        self.values.insert(
            key,
            DynamicProperty {
                value,
                enumerable: false,
            },
        );
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
