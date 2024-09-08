use std::borrow::Borrow;
use std::cell::Cell;
use std::default::Default;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::ops::DerefMut;

use gc_arena::{Collect, Gc, GcWeak, Mutation};
use hashbrown::HashSet;

/// A set holding weakly to its elements. Mostly useful for implementing interning tables.
///
/// Stale entries get regularly cleaned up in response to memory pressure:
/// - in the tracing phase of each GC cycle;
/// - upon insertion when the set is at capacity.
pub struct WeakSet<'gc, T: 'gc> {
    // Note that `GcWeak<T>` does not implement `Hash`, so the `RawTable`
    // API is used for lookups and insertions.
    table: CollectCell<'gc, HashSet<GcWeak<'gc, T>, ()>>,
    hasher: fnv::FnvBuildHasher,
}

impl<'gc, T: 'gc> WeakSet<'gc, T> {
    pub fn new() -> Self {
        Self {
            table: Default::default(),
            hasher: Default::default(),
        }
    }
}

impl<'gc, T: 'gc> Default for WeakSet<'gc, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc, T: Hash + 'gc> WeakSet<'gc, T> {
    /// Finds the given key in the map.
    pub fn get<Q>(&self, mc: &Mutation<'gc>, key: &Q) -> Option<Gc<'gc, T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let raw = self.table.as_ref(mc).raw_table();
        let hash = self.hasher.hash_one(key);
        let mut found = None;
        let _ = raw.find(hash, |(weak, _)| {
            if let Some(strong) = weak.upgrade(mc) {
                if (*strong).borrow() == key {
                    found = Some(strong);
                    return true;
                }
            }
            false
        });
        found
    }

    /// Finds the given key in the map, and return its entry.
    /// This also cleans up stale buckets found along the way.
    /// TODO: add proper entry API?
    pub fn entry<'a, Q>(&'a mut self, mc: &'a Mutation<'gc>, key: &Q) -> Entry<'a, 'gc, T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let raw = self.table.as_mut().raw_table_mut();
        let hash = self.hasher.hash_one(key);

        // SAFETY: the iterator doesn't outlive the `HashSet`.
        for bucket in unsafe { raw.iter_hash(hash) } {
            // SAFETY: `iter_hash` only returns occupied buckets.
            let weak = unsafe { bucket.as_ref().0 };

            if let Some(strong) = weak.upgrade(mc) {
                // The entry matches, return it.
                if (*strong).borrow() == key {
                    return Entry::Occupied(OccupiedEntry {
                        _table: PhantomData,
                        value: strong,
                    });
                }
            } else {
                // The entry is stale, delete it.
                // SAFETY: the entry has already been yielded by the iterator.
                unsafe { raw.erase(bucket) };
            }
        }

        Entry::Vacant(VacantEntry {
            table: self,
            mc,
            hash,
        })
    }

    pub fn insert_unique_unchecked(&mut self, mc: &Mutation<'gc>, key: Gc<'gc, T>) -> Gc<'gc, T> {
        let hash = self.hasher.hash_one(&*key);
        self.insert_fresh(mc, hash, key);
        key
    }

    fn insert_fresh<'a>(
        &'a mut self,
        mc: &'a Mutation<'gc>,
        hash: u64,
        key: Gc<'gc, T>,
    ) -> OccupiedEntry<'a, 'gc, T> {
        let entry = (Gc::downgrade(key), ());

        let raw = self.table.as_mut().raw_table_mut();

        if raw.try_insert_no_grow(hash, entry).is_err() {
            self.prune_and_grow(mc);
            let raw = self.table.as_mut().raw_table_mut();
            raw.try_insert_no_grow(hash, entry)
                .expect("failed to grow table");
        }

        OccupiedEntry {
            _table: PhantomData,
            value: key,
        }
    }

    /// Prune stale entries and/or resize the table to ensure at least one extra entry can be added.
    #[cold]
    fn prune_and_grow(&mut self, mc: &Mutation<'gc>) {
        let table = self.table.as_mut();

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
                    Some(strong) => self.hasher.hash_one(&*strong),
                    None => unreachable!("unexpected stale entry"),
                });
        }
    }
}

unsafe impl<'gc, T> Collect for WeakSet<'gc, T> {
    fn trace(&self, cc: &gc_arena::Collection) {
        // Prune entries known to be dead.
        // Safe, as we never pick up new GC pointers from outside this allocation.
        let mut guard = unsafe { self.table.steal_for_trace() };
        guard.retain(|weak| {
            let keep = !weak.is_dropped(cc);
            if keep {
                // NOTE: The explicit dereference is necessary to not
                // use the no-op `Collect` impl on references.
                (*weak).trace(cc);
            }
            keep
        });
    }
}

pub enum Entry<'a, 'gc, T> {
    Vacant(VacantEntry<'a, 'gc, T>),
    Occupied(OccupiedEntry<'a, 'gc, T>),
}

impl<'a, 'gc, T: Hash> Entry<'a, 'gc, T> {
    pub fn get(&self) -> Option<Gc<'gc, T>> {
        match self {
            Self::Vacant(_) => None,
            Self::Occupied(e) => Some(e.get()),
        }
    }

    /// Sets the value of this entry, if it's empty.
    ///
    /// The given value should have the same hash as the value initially used
    /// to construct this [`Entry`].
    pub fn or_insert(self, key: impl FnOnce() -> Gc<'gc, T>) -> OccupiedEntry<'a, 'gc, T> {
        match self {
            Self::Vacant(e) => e.insert(key()),
            Self::Occupied(e) => e,
        }
    }
}

pub struct VacantEntry<'a, 'gc, T> {
    mc: &'a Mutation<'gc>,
    table: &'a mut WeakSet<'gc, T>,
    hash: u64,
}

impl<'a, 'gc, T: Hash> VacantEntry<'a, 'gc, T> {
    /// Sets the value of this entry.
    ///
    /// The given value should have the same hash as the value initially used
    /// to construct this [`VacantEntry`].
    pub fn insert(self, key: Gc<'gc, T>) -> OccupiedEntry<'a, 'gc, T> {
        self.table.insert_fresh(self.mc, self.hash, key)
    }
}

pub struct OccupiedEntry<'a, 'gc, T> {
    _table: PhantomData<&'a mut WeakSet<'gc, T>>,
    value: Gc<'gc, T>,
}

impl<'a, 'gc, T: Hash> OccupiedEntry<'a, 'gc, T> {
    pub fn get(&self) -> Gc<'gc, T> {
        self.value
    }
}

/// Small utility to work-around the fact that `Collect::trace` only takes `&self`.
#[derive(Default)]
struct CollectCell<'gc, T> {
    inner: Cell<T>,
    _marker: PhantomData<Gc<'gc, T>>,
}

impl<'gc, T> CollectCell<'gc, T> {
    #[inline(always)]
    fn as_ref<'a>(&'a self, _mc: &Mutation<'gc>) -> &'a T {
        unsafe { &*self.inner.as_ptr() }
    }

    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// SAFETY: must be called inside a `Collect::trace` function.
    ///
    /// An alternative would be to require a `&gc_arena::Collection` argument, but this is
    /// still unsound in presence of nested arenas (preventing this would require a `'gc`
    /// lifetime on `&gc_arena::Collection` and `Collect`):
    ///
    /// ```rs,ignore
    /// fn trace(&self, cc: &gc_arena::Collection) {
    ///     rootless_arena(|mc| {
    ///         let cell = CollectCell::<i32>::default();
    ///         let borrow: &i32 = dbg!(cell.as_ref(mc)); // 0
    ///         *cell.steal_for_trace(cc) = 1;
    ///         dbg!(borrow); // 1 - oh no!
    ///     });
    /// }
    /// ```
    #[inline(always)]
    unsafe fn steal_for_trace(&self) -> impl DerefMut<Target = T> + '_
    where
        T: Default,
    {
        let cell = &self.inner;
        scopeguard::guard(cell.take(), |stolen| cell.set(stolen))
    }
}
