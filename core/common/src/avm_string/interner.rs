use super::avm_string::AvmString;
use super::common::CommonStrings;
use super::repr::AvmStringRepr;

use core::fmt;
use gc_arena::collect::Trace;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use hashbrown::HashTable;
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
            Entry::Occupied(occupied) => AvmAtom(occupied.get()),
            Entry::Vacant(vacant) => AvmAtom(vacant.insert(mc, f(s))),
        }
    }

    #[must_use]
    pub(super) fn get(&mut self, mc: &Mutation<'gc>, s: &WStr) -> Option<AvmAtom<'gc>> {
        match self.interned.entry(mc, s) {
            Entry::Occupied(occupied) => Some(AvmAtom(occupied.get())),
            Entry::Vacant(_) => None,
        }
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
    // Note that `GcWeak<T>` does not implement `Hash`, so `HashTable`
    // is used for lookups and insertions with explicit hashing.
    // The `RefCell` is only used to get mutable access in `Collect::trace`
    table: RefCell<HashTable<GcWeak<'gc, T>>>,
    hasher: fnv::FnvBuildHasher,
}

enum Entry<'a, 'gc, T: 'gc> {
    Occupied(OccupiedEntry<'gc, T>),
    Vacant(VacantEntry<'a, 'gc, T>),
}

struct OccupiedEntry<'gc, T: 'gc>(Gc<'gc, T>);

impl<'gc, T: 'gc> OccupiedEntry<'gc, T> {
    fn get(&self) -> Gc<'gc, T> {
        self.0
    }
}

struct VacantEntry<'a, 'gc, T: 'gc> {
    set: &'a mut WeakSet<'gc, T>,
    hash: u64,
}

impl<'a, 'gc, T: Hash + 'gc> VacantEntry<'a, 'gc, T> {
    fn insert(self, mc: &Mutation<'gc>, key: Gc<'gc, T>) -> Gc<'gc, T> {
        self.set.insert_fresh(mc, self.hash, key)
    }
}

impl<'gc, T: Hash + 'gc> WeakSet<'gc, T> {
    fn hash<K: Hash + ?Sized>(build_hasher: &impl BuildHasher, key: &K) -> u64 {
        build_hasher.hash_one(key)
    }

    fn weak_hasher<'a>(
        hasher: &'a fnv::FnvBuildHasher,
        mc: &'a Mutation<'gc>,
    ) -> impl Fn(&GcWeak<'gc, T>) -> u64 + 'a {
        move |w| match w.upgrade(mc) {
            Some(strong) => Self::hash(hasher, &*strong),
            None => 0,
        }
    }

    fn entry<Q>(&mut self, mc: &Mutation<'gc>, key: &Q) -> Entry<'_, 'gc, T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = Self::hash(&self.hasher, key);
        let table = self.table.get_mut();

        loop {
            match table.find_entry(hash, |weak| {
                weak.upgrade(mc).is_none_or(|s| (*s).borrow() == key)
            }) {
                Ok(occupied) => {
                    if let Some(strong) = occupied.get().upgrade(mc) {
                        return Entry::Occupied(OccupiedEntry(strong));
                    } else {
                        occupied.remove();
                    }
                }
                Err(_) => return Entry::Vacant(VacantEntry { set: self, hash }),
            }
        }
    }

    /// Inserts a new key in the set.
    /// The key must not already exist.
    fn insert_fresh_no_hash(&mut self, mc: &Mutation<'gc>, key: Gc<'gc, T>) -> Gc<'gc, T> {
        let hash = Self::hash(&self.hasher, &key);

        self.insert_fresh(mc, hash, key)
    }

    /// Inserts a new key in the set.
    /// The key must not already exist, and `hash` must be its hash.
    fn insert_fresh(&mut self, mc: &Mutation<'gc>, hash: u64, key: Gc<'gc, T>) -> Gc<'gc, T> {
        let weak = Gc::downgrade(key);

        let table = self.table.get_mut();

        if table.len() >= table.capacity() {
            self.prune_and_grow(mc);
        }

        self.table
            .get_mut()
            .insert_unique(hash, weak, Self::weak_hasher(&self.hasher, mc));

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
            table.reserve(extra, Self::weak_hasher(&self.hasher, mc));
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
