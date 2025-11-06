use super::avm_string::AvmString;
use super::common::CommonStrings;
use super::repr::AvmStringRepr;

use core::fmt;
use gc_arena::collect::Trace;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use hashbrown::HashSet;
use ruffle_wstr::WStr;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::Deref;

// An interned `AvmString`, with fast by-pointer equality and hashing.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct AvmAtom<'gc>(pub(super) Gc<'gc, AvmStringRepr<'gc>>);

impl PartialEq for AvmAtom<'_> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl Eq for AvmAtom<'_> {}

impl Hash for AvmAtom<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Gc::as_ptr(self.0).hash(state);
    }
}

impl fmt::Debug for AvmAtom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_wstr(), f)
    }
}

impl fmt::Display for AvmAtom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_wstr(), f)
    }
}

impl AvmAtom<'_> {
    pub fn as_wstr(&self) -> &WStr {
        &self.0
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct AvmStringInterner<'gc> {
    interned: WeakSet<'gc, AvmStringRepr<'gc>>,

    /// Strings used across both AVMs and in core code.
    pub(super) common: CommonStrings<'gc>,
}

impl<'gc> AvmStringInterner<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        let mut interned = WeakSet::default();

        let common = CommonStrings::new(
            // We can't use `Self::intern_static` because we don't have a Self yet.
            #[inline(never)]
            |s: &'static [u8]| {
                let wstr = WStr::from_units(s);
                let repr = AvmStringRepr::from_raw_static(wstr, true);
                AvmAtom(interned.insert_fresh_no_hash(mc, Gc::new(mc, repr)))
            },
        );

        Self { interned, common }
    }

    /// The string returned by `f` should be interned, and equivalent to `s`.
    pub(super) fn intern_inner<S, F>(&mut self, mc: &Mutation<'gc>, s: S, f: F) -> AvmAtom<'gc>
    where
        S: Deref<Target = WStr>,
        F: FnOnce(S) -> Gc<'gc, AvmStringRepr<'gc>>,
    {
        match self.interned.entry(mc, s.deref()) {
            (Some(atom), _) => AvmAtom(atom),
            (None, h) => {
                let atom = self.interned.insert_fresh(mc, h, f(s));
                AvmAtom(atom)
            }
        }
    }

    #[must_use]
    pub(super) fn get(&mut self, mc: &Mutation<'gc>, s: &WStr) -> Option<AvmAtom<'gc>> {
        self.interned.get(mc, s).map(AvmAtom)
    }

    #[must_use]
    pub(super) fn substring(
        &self,
        mc: &Mutation<'gc>,
        s: AvmString<'gc>,
        start_index: usize,
        end_index: usize,
    ) -> AvmString<'gc> {
        // TODO: return original string if full range

        // It's assumed that start<=end. This is tested later via a range check.
        if start_index == end_index {
            self.common.str_.into() // this is the empty string
        } else if end_index == start_index + 1
            && let Some(c) = s.get(start_index)
            && let Some(s) = self.common.ascii_chars.get(c as usize)
        {
            (*s).into()
        } else {
            AvmString::substring(mc, s, start_index, end_index)
        }
    }
}

/// A set holding weakly to its elements.
///
/// Stale entries get regularly cleaned up in response to memory pressure:
/// - in the tracing phase of each GC cycle;
/// - upon insertion when the set is at capacity.
#[derive(Default)]
struct WeakSet<'gc, T: 'gc> {
    // Note that `GcWeak<T>` does not implement `Hash`, so the `RawTable`
    // API is used for lookups and insertions.
    // The `RefCell` is only used to get mutable access in `Collect::trace`
    table: RefCell<HashSet<GcWeak<'gc, T>>>,
    hasher: fnv::FnvBuildHasher,
}

impl<'gc, T: Hash + 'gc> WeakSet<'gc, T> {
    fn hash<K: Hash + ?Sized>(build_hasher: &impl BuildHasher, key: &K) -> u64 {
        build_hasher.hash_one(key)
    }

    /// Finds the given key in the map.
    /// This takes `&mut self` to be able to clean dead entries (and to avoid a `RefCell` check).
    fn get<Q>(&mut self, mc: &Mutation<'gc>, key: &Q) -> Option<Gc<'gc, T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.entry(mc, key).0
    }

    /// Finds the given key in the map, and return its and its hash.
    /// This also cleans up stale buckets found along the way.
    /// TODO: add proper entry API?
    fn entry<Q>(&mut self, mc: &Mutation<'gc>, key: &Q) -> (Option<Gc<'gc, T>>, u64)
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let raw = self.table.get_mut().raw_table_mut();
        let hash = Self::hash(&self.hasher, key);

        // SAFETY: the iterator doesn't outlive the `HashSet`.
        for bucket in unsafe { raw.iter_hash(hash) } {
            // SAFETY: `iter_hash` only returns occupied buckets.
            let weak = unsafe { bucket.as_ref().0 };

            if let Some(strong) = weak.upgrade(mc) {
                // The entry matches, return it.
                if (*strong).borrow() == key {
                    return (Some(strong), hash);
                }
            } else {
                // The entry is stale, delete it.
                // SAFETY: the entry has already been yielded by the iterator.
                unsafe { raw.erase(bucket) };
            }
        }

        (None, hash)
    }

    /// Inserts a new key in the set.
    /// The key must not already exist
    /// TODO: add proper entry API?
    fn insert_fresh_no_hash(&mut self, mc: &Mutation<'gc>, key: Gc<'gc, T>) -> Gc<'gc, T> {
        let hash = Self::hash(&self.hasher, &key);
        self.insert_fresh(mc, hash, key)
    }

    /// Inserts a new key in the set.
    /// The key must not already exist, and `hash` must be its hash.
    /// TODO: add proper entry API?
    fn insert_fresh(&mut self, mc: &Mutation<'gc>, hash: u64, key: Gc<'gc, T>) -> Gc<'gc, T> {
        let entry = (Gc::downgrade(key), ());

        let raw = self.table.get_mut().raw_table_mut();

        if raw.try_insert_no_grow(hash, entry).is_err() {
            self.prune_and_grow(mc);
            let raw = self.table.get_mut().raw_table_mut();
            raw.try_insert_no_grow(hash, entry)
                .expect("failed to grow table");
        }

        key
    }

    /// Prune stale entries and/or resize the table to ensure at least one extra entry can be added.
    #[cold]
    fn prune_and_grow(&mut self, mc: &Mutation<'gc>) {
        let table = self.table.get_mut();

        // We *really* don't want to reallocate, so try to prune dead references first.
        let all = table.len();
        table.retain(|weak| weak.upgrade(mc).is_some());
        let remaining = table.len();

        // Only reallocate if few entries were pruned.
        if remaining >= all / 2 {
            let extra = all - remaining + 1;
            table
                .raw_table_mut()
                .reserve(extra, |(weak, _)| match weak.upgrade(mc) {
                    Some(strong) => Self::hash(&self.hasher, &*strong),
                    None => unreachable!("unexpected stale entry"),
                });
        }
    }
}

unsafe impl<'gc, T> Collect<'gc> for WeakSet<'gc, T> {
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        // Prune entries known to be dead.
        // Safe, as we never pick up new GC pointers from outside this allocation.
        let mut table = self.table.borrow_mut();
        table.retain(|weak| {
            let keep = !weak.is_dropped();
            if keep {
                cc.trace(weak);
            }
            keep
        });
    }
}
