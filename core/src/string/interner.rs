use core::fmt;
use std::borrow::{Borrow, Cow};
use std::cell::Cell;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::ops::DerefMut;

use gc_arena::{Collect, Gc, GcWeak, Mutation};
use hashbrown::HashSet;

use crate::string::{AvmString, AvmStringRepr, WStr, WString};

// An interned `AvmString`, with fast by-pointer equality and hashing.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct AvmAtom<'gc>(pub(super) Gc<'gc, AvmStringRepr<'gc>>);

impl<'gc> PartialEq for AvmAtom<'gc> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl<'gc> Eq for AvmAtom<'gc> {}

impl<'gc> Hash for AvmAtom<'gc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Gc::as_ptr(self.0).hash(state);
    }
}

impl<'gc> fmt::Debug for AvmAtom<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_wstr(), f)
    }
}

impl<'gc> fmt::Display for AvmAtom<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_wstr(), f)
    }
}

impl<'gc> AvmAtom<'gc> {
    pub fn as_wstr(&self) -> &WStr {
        &self.0
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct AvmStringInterner<'gc> {
    interned: WeakSet<'gc, AvmStringRepr<'gc>>,

    empty: Gc<'gc, AvmStringRepr<'gc>>,
    chars: [Gc<'gc, AvmStringRepr<'gc>>; 128],
}

impl<'gc> AvmStringInterner<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        let mut interned = WeakSet::default();

        let mut intern_from_static = |s: &[u8]| {
            let ch = Self::alloc(mc, Cow::Borrowed(WStr::from_units(s)));
            interned.insert_fresh_no_hash(mc, ch)
        };

        let empty = intern_from_static(b"");

        let mut chars = [empty; 128];
        for (i, elem) in chars.iter_mut().enumerate() {
            *elem = intern_from_static(&[i as u8]);
        }

        Self {
            interned,
            empty,
            chars,
        }
    }

    fn alloc(mc: &Mutation<'gc>, s: Cow<'_, WStr>) -> Gc<'gc, AvmStringRepr<'gc>> {
        // note: the string is already marked as interned
        let repr = AvmStringRepr::from_raw(s.into_owned(), true);
        Gc::new(mc, repr)
    }

    #[must_use]
    pub fn intern_wstr<'a, S>(&mut self, mc: &Mutation<'gc>, s: S) -> AvmAtom<'gc>
    where
        S: Into<Cow<'a, WStr>>,
    {
        let s = s.into();
        let atom = match self.interned.entry(mc, s.as_ref()) {
            (Some(atom), _) => atom,
            (None, h) => self.interned.insert_fresh(mc, h, Self::alloc(mc, s)),
        };

        AvmAtom(atom)
    }

    #[must_use]
    pub fn get(&self, mc: &Mutation<'gc>, s: &WStr) -> Option<AvmAtom<'gc>> {
        self.interned.get(mc, s).map(AvmAtom)
    }

    #[must_use]
    pub fn intern(&mut self, mc: &Mutation<'gc>, s: AvmString<'gc>) -> AvmAtom<'gc> {
        if let Some(atom) = s.as_interned() {
            return atom;
        }

        let atom = match self.interned.entry(mc, s.as_wstr()) {
            (Some(atom), _) => atom,
            (None, h) => {
                let repr = s.to_fully_owned(mc);
                repr.mark_interned();
                self.interned.insert_fresh(mc, h, repr)
            }
        };

        AvmAtom(atom)
    }

    #[must_use]
    pub fn empty(&self) -> AvmString<'gc> {
        self.empty.into()
    }

    #[must_use]
    pub fn get_char(&self, mc: &Mutation<'gc>, c: u16) -> AvmString<'gc> {
        if let Some(s) = self.chars.get(c as usize) {
            (*s).into()
        } else {
            AvmString::new(mc, WString::from_unit(c))
        }
    }

    #[must_use]
    pub fn substring(
        &self,
        mc: &Mutation<'gc>,
        s: AvmString<'gc>,
        start_index: usize,
        end_index: usize,
    ) -> AvmString<'gc> {
        // TODO: return original string if full range

        // It's assumed that start<=end. This is tested later via a range check.
        if start_index == end_index {
            return self.empty.into();
        }
        if end_index == start_index + 1 {
            if let Some(c) = s.get(start_index) {
                if let Some(s) = self.chars.get(c as usize) {
                    return (*s).into();
                }
            }
        }
        AvmString::substring(mc, s, start_index, end_index)
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
    table: CollectCell<'gc, HashSet<GcWeak<'gc, T>>>,
    hasher: fnv::FnvBuildHasher,
}

impl<'gc, T: Hash + 'gc> WeakSet<'gc, T> {
    fn hash<K: Hash + ?Sized>(build_hasher: &impl BuildHasher, key: &K) -> u64 {
        build_hasher.hash_one(key)
    }

    /// Finds the given key in the map.
    fn get<Q>(&self, mc: &Mutation<'gc>, key: &Q) -> Option<Gc<'gc, T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let raw = self.table.as_ref(mc).raw_table();
        let hash = Self::hash(&self.hasher, key);
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

    /// Finds the given key in the map, and return its and its hash.
    /// This also cleans up stale buckets found along the way.
    /// TODO: add proper entry API?
    fn entry<Q>(&mut self, mc: &Mutation<'gc>, key: &Q) -> (Option<Gc<'gc, T>>, u64)
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let raw = self.table.as_mut().raw_table_mut();
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

        let raw = self.table.as_mut().raw_table_mut();

        if raw.try_insert_no_grow(hash, entry).is_err() {
            self.prune_and_grow(mc);
            let raw = self.table.as_mut().raw_table_mut();
            raw.try_insert_no_grow(hash, entry)
                .expect("failed to grow table");
        }

        key
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
                    Some(strong) => Self::hash(&self.hasher, &*strong),
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
