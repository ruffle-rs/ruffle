use std::borrow::{Borrow, Cow};
use std::cell::Cell;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::ops::DerefMut;

use gc_arena::{Collect, CollectionContext, Gc, GcWeak, MutationContext};
use hashbrown::raw::{Bucket, RawTable};

use crate::string::{AvmString, OwnedWStr, WStr};

// An interned `AvmString`, with fast by-pointer equality and hashing.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct AvmAtom<'gc>(Gc<'gc, OwnedWStr>);

impl<'gc> PartialEq for AvmAtom<'gc> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl<'gc> PartialEq<AvmString<'gc>> for AvmAtom<'gc> {
    fn eq(&self, other: &AvmString<'gc>) -> bool {
        if let Some(atom) = other.as_interned() {
            *self == atom
        } else {
            self.as_wstr() == other.as_wstr()
        }
    }
}

impl<'gc> PartialEq<AvmAtom<'gc>> for AvmString<'gc> {
    #[inline(always)]
    fn eq(&self, other: &AvmAtom<'gc>) -> bool {
        PartialEq::eq(other, self)
    }
}

impl<'gc> Eq for AvmAtom<'gc> {}

impl<'gc> Hash for AvmAtom<'gc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Gc::as_ptr(self.0).hash(state);
    }
}

impl<'gc> AvmAtom<'gc> {
    pub fn as_wstr(&self) -> &WStr {
        &self.0
    }

    pub(super) fn as_owned(self) -> Gc<'gc, OwnedWStr> {
        self.0
    }
}

#[derive(Collect, Default)]
#[collect(no_drop)]
pub struct AvmStringInterner<'gc> {
    interned: WeakSet<'gc, OwnedWStr>,
}

impl<'gc> AvmStringInterner<'gc> {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc(mc: MutationContext<'gc, '_>, s: Cow<'_, WStr>) -> Gc<'gc, OwnedWStr> {
        Gc::allocate(mc, OwnedWStr(s.into_owned()))
    }

    #[must_use]
    pub fn intern_wstr<'a, S>(&mut self, mc: MutationContext<'gc, '_>, s: S) -> AvmAtom<'gc>
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
    pub fn get(&self, mc: MutationContext<'gc, '_>, s: &WStr) -> Option<AvmAtom<'gc>> {
        self.interned.get(mc, s).map(AvmAtom)
    }

    #[must_use]
    pub fn intern(&mut self, mc: MutationContext<'gc, '_>, s: AvmString<'gc>) -> AvmAtom<'gc> {
        if let Some(atom) = s.as_interned() {
            return atom;
        }

        let atom = match self.interned.entry(mc, s.as_wstr()) {
            (Some(atom), _) => atom,
            (None, h) => self.interned.insert_fresh(mc, h, s.to_owned(mc)),
        };

        AvmAtom(atom)
    }
}

/// A set holding weakly to its elements.
///
/// Stale entries get regularly cleaned up in response to memory pressure:
/// - in the tracing phase of each GC cycle;
/// - upon insertion when the set is at capacity.
#[derive(Default)]
struct WeakSet<'gc, T: 'gc> {
    // Cannot use `HashSet` here, as `GcWeak` cannot implement `Hash`.
    raw: CollectCell<'gc, RawTable<GcWeak<'gc, T>>>,
    hasher: fnv::FnvBuildHasher,
}

struct FindResult<'gc, T: 'gc> {
    value: Option<Gc<'gc, T>>,
    stale: Option<Bucket<GcWeak<'gc, T>>>,
}

impl<'gc, T: 'gc> WeakSet<'gc, T> {
    fn hash<K: Hash + ?Sized>(build_hasher: &impl BuildHasher, key: &K) -> u64 {
        let mut hasher = build_hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Finds a live, matching entry with a given hash.
    /// Returns both the matching entry and the first stale bucket found along the way.
    fn find_inner(
        &self,
        mc: MutationContext<'gc, '_>,
        hash: u64,
        mut eq: impl FnMut(&T) -> bool,
    ) -> FindResult<'gc, T> {
        let raw = self.raw.as_ref(mc);
        let mut result = FindResult {
            value: None,
            stale: None,
        };

        // SAFETY: `iter_hash` doesn't outlive `raw`, and only returns full buckets.
        unsafe {
            for bucket in raw.iter_hash(hash) {
                match bucket.as_ref().upgrade(mc) {
                    Some(strong) if eq(&strong) => {
                        result.value = Some(strong);
                        break;
                    }
                    None if result.stale.is_none() => {
                        result.stale = Some(bucket);
                    }
                    _ => (),
                }
            }
        }

        result
    }

    /// Finds the given key in the map.
    fn get<Q>(&self, mc: MutationContext<'gc, '_>, key: &Q) -> Option<Gc<'gc, T>>
    where
        T: Borrow<Q> + Hash,
        Q: Hash + Eq + ?Sized,
    {
        let hash = Self::hash(&self.hasher, key);
        let result = self.find_inner(mc, hash, |strong| strong.borrow() == key);
        result.value
    }

    /// Finds the given key in the map, and return its and its hash.
    /// TODO: add proper entry API?
    fn entry<Q>(&mut self, mc: MutationContext<'gc, '_>, key: &Q) -> (Option<Gc<'gc, T>>, u64)
    where
        T: Borrow<Q> + Hash,
        Q: Hash + Eq + ?Sized,
    {
        let hasher = &self.hasher;
        let hash = Self::hash(hasher, key);
        let result = self.find_inner(mc, hash, |strong| strong.borrow() == key);

        // Clear any stale bucket found; this ensures that reinserting
        // a freshly pruned key does not grow the table.
        if let Some(stale) = result.stale {
            unsafe { self.raw.as_mut().erase(stale) }
        }

        (result.value, hash)
    }

    /// Inserts a new key in the set.
    /// The key must not already exist, and `hash` must be its hash.
    /// TODO: add proper entry API?
    fn insert_fresh(
        &mut self,
        mc: MutationContext<'gc, '_>,
        hash: u64,
        key: Gc<'gc, T>,
    ) -> Gc<'gc, T>
    where
        T: Hash,
    {
        let weak = Gc::downgrade(key);
        let raw = self.raw.as_mut();
        let hasher = &self.hasher;

        if raw.try_insert_no_grow(hash, weak).is_err() {
            Self::prune_and_grow(raw, |w| w.upgrade(mc), |k| Self::hash(hasher, &**k));
            raw.try_insert_no_grow(hash, weak)
                .expect("failed to grow table");
        }

        key
    }

    /// Prune stale entries and resize the table to ensure at least one extra entry can be added.
    #[cold]
    fn prune_and_grow<K, B>(
        raw: &mut RawTable<B>,
        upgrade: impl Fn(&B) -> Option<K>,
        hasher: impl Fn(&K) -> u64,
    ) {
        // We *really* don't want to reallocate, so try to prune dead references first.
        let all = raw.len();
        Self::retain(raw, |b| upgrade(b).is_some());
        let remaining = raw.len();

        // Only reallocate if few entries were pruned.
        if remaining >= all / 2 {
            raw.reserve(all - remaining + 1, |b| match upgrade(b) {
                Some(k) => hasher(&k),
                None => unreachable!("unexpected stale entry"),
            })
        }
    }

    /// Filters the entries of a raw table.
    fn retain<B>(raw: &mut RawTable<B>, mut f: impl FnMut(&mut B) -> bool) {
        // SAFETY: `iter` doesn't outlive `raw`, and only return full buckets.
        unsafe {
            for bucket in raw.iter() {
                if !f(bucket.as_mut()) {
                    raw.erase(bucket);
                }
            }
        }
    }
}

unsafe impl<'gc, T> Collect for WeakSet<'gc, T> {
    fn trace(&self, cc: CollectionContext) {
        // Prune entries known to be dead.
        // Safe, as we never pick up new GC pointers from outside this allocation.
        let mut guard = unsafe { self.raw.steal_for_trace() };
        Self::retain(&mut *guard, |weak| {
            let keep = !weak.is_dropped();
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
    fn as_ref<'a>(&'a self, _mc: MutationContext<'gc, '_>) -> &'a T {
        unsafe { &*self.inner.as_ptr() }
    }

    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// SAFETY: must be called inside a `Collect::trace` function.
    ///
    /// An alternative would be to require a `CollectionContext` argument, but this is
    /// still unsound in presence of nested arenas (preventing this would require a `'gc`
    /// lifetime on `CollectionContext` and `Collect`):
    ///
    /// ```rs,ignore
    /// fn trace(&self, cc: CollectionContext) {
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
