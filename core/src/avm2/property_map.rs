//! Property map

use crate::avm2::AvmString;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use fnv::FnvBuildHasher;
use gc_arena::Collect;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::mem::swap;

/// Type which represents named properties on an object.
///
/// This type exposes interfaces akin to `HashMap<QName<'gc>, V>`, and is
/// intended to serve as a drop-in replacement optimized for objects where few
/// properties have overlapping local names. However, we have made slight
/// changes to the API in the following cases:
///
///  * Iterators return tuples of namespace, local-name, and value; rather than
///    a qualified name and value pair.
///  * Only `HashMap` methods and traits that we need are implemented.
///
/// The internal structure of the `PropertyMap` technically allows storage of
/// multiple values per `QName`. It's implementation enforces the invariant
/// that each `QName` only have one associated `V`.
#[derive(Clone, Debug)]
pub struct PropertyMap<'gc, V>(
    HashMap<AvmString<'gc>, SmallVec<[(Namespace<'gc>, V); 2]>, FnvBuildHasher>,
);

unsafe impl<V> Collect for PropertyMap<'_, V>
where
    V: Collect,
{
    #[inline]
    fn trace(&self, cc: &gc_arena::Collection) {
        for (key, value) in self.0.iter() {
            key.trace(cc);
            for (ns, v) in value.iter() {
                ns.trace(cc);
                v.trace(cc);
            }
        }
    }
}

impl<V> Default for PropertyMap<'_, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc, V> PropertyMap<'gc, V> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get(&self, name: QName<'gc>) -> Option<&V> {
        self.0.get(&name.local_name()).iter().find_map(|v| {
            v.iter()
                .filter(|(n, _)| n.matches_ns(name.namespace()))
                .map(|(_, v)| v)
                .next()
        })
    }

    pub fn get_for_multiname(&self, name: &Multiname<'gc>) -> Option<&V> {
        if name.has_lazy_component() {
            unreachable!("Lookup on lazy Multiname should never happen ({:?})", name);
        }
        if let Some(local_name) = name.local_name() {
            self.0.get(&local_name).iter().find_map(|v| {
                v.iter()
                    .filter(|(n, _)| name.namespace_set().iter().any(|ns| n.matches_ns(*ns)))
                    .map(|(_, v)| v)
                    .next()
            })
        } else {
            None
        }
    }

    pub fn get_with_ns_for_multiname(&self, name: &Multiname<'gc>) -> Option<(Namespace<'gc>, &V)> {
        if name.has_lazy_component() {
            unreachable!("Lookup on lazy Multiname should never happen ({:?})", name);
        }
        if let Some(local_name) = name.local_name() {
            self.0.get(&local_name).iter().find_map(|v| {
                v.iter()
                    .filter(|(n, _)| name.namespace_set().iter().any(|ns| n.matches_ns(*ns)))
                    .map(|(ns, v)| (*ns, v))
                    .next()
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, name: QName<'gc>) -> Option<&mut V> {
        if let Some(bucket) = self.0.get_mut(&name.local_name()) {
            if let Some((_, old_value)) = bucket
                .iter_mut()
                .find(|(n, _)| n.matches_ns(name.namespace()))
            {
                return Some(old_value);
            }
        }

        None
    }

    pub fn contains_key(&self, name: QName<'gc>) -> bool {
        self.0
            .get(&name.local_name())
            .iter()
            .any(|v| v.iter().any(|(n, _)| n.matches_ns(name.namespace())))
    }

    pub fn iter(&self) -> impl Iterator<Item = (AvmString<'gc>, Namespace<'gc>, &V)> {
        self.0
            .iter()
            .flat_map(|(k, vs)| vs.iter().map(|(ns, v)| (*k, *ns, v)))
    }

    pub fn insert(&mut self, name: QName<'gc>, mut value: V) -> Option<V> {
        let bucket = self.0.entry(name.local_name()).or_default();

        if let Some((_, old_value)) = bucket
            .iter_mut()
            .find(|(n, _)| n.matches_ns(name.namespace()))
        {
            swap(old_value, &mut value);

            Some(value)
        } else {
            bucket.push((name.namespace(), value));

            None
        }
    }

    pub fn insert_with_namespace(
        &mut self,
        ns: Namespace<'gc>,
        name: AvmString<'gc>,
        mut value: V,
    ) -> Option<V> {
        let bucket = self.0.entry(name).or_default();

        if let Some((_, old_value)) = bucket.iter_mut().find(|(n, _)| n.matches_ns(ns)) {
            swap(old_value, &mut value);

            Some(value)
        } else {
            bucket.push((ns, value));

            None
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, name: QName<'gc>) -> Option<V> {
        let bucket = self.0.get_mut(&name.local_name());

        if let Some(bucket) = bucket {
            let position = bucket
                .iter_mut()
                .enumerate()
                .find(|(_, (n, _))| n.matches_ns(name.namespace()));
            if let Some((position, _)) = position {
                return Some(bucket.remove(position).1);
            }
        }

        None
    }
}
