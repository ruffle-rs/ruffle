use core::cell::Cell;
use core::cmp::Ordering;
use core::fmt;

use gc_arena::{Collect, CollectionContext};

use super::{Unlock, Write};

/// GC-aware [`Cell`], only allowing operations compatible with no write barrier.
#[repr(transparent)]
#[derive(Default)]
pub struct Lock<T: ?Sized> {
    value: Cell<T>,
}

impl<T> Lock<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: Cell::new(value),
        }
    }

    #[inline]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.value.get()
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    #[inline]
    pub fn take(&self) -> T
    where
        T: Default,
    {
        // Despite mutating the contained value, this doesn't need a write barrier,
        // as, thanks to lifetime parametricity, a `Default::default()` cannot ever
        // safely obtain a `MutationContext` or other external `Gc` pointers.
        self.value.take()
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    #[inline]
    pub fn unlock(this: &Write<Self>) -> &Cell<T> {
        &this.value
    }
}

unsafe impl<T: Collect + Copy> Collect for Lock<T> {
    #[inline]
    fn needs_trace() -> bool {
        T::needs_trace()
    }

    #[inline]
    fn trace(&self, cc: CollectionContext) {
        // Use a guard that writes back the value in case the 'trace' call mutates it.
        let cell = &self.value;
        let guard = scopeguard::guard(cell.get(), |copied| cell.set(copied));
        T::trace(&guard, cc);
    }
}

impl<T: ?Sized> Unlock for Lock<T> {
    type Unlocked = Cell<T>;

    #[inline]
    unsafe fn unlock_unchecked(&self) -> &Cell<T> {
        &self.value
    }
}

impl<T> From<T> for Lock<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: fmt::Debug + Copy> fmt::Debug for Lock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lock")
            .field("value", &self.value.get())
            .finish()
    }
}

// Can't use `#[derive]` for these impls because of the non-trivial bounds.

impl<T: Copy> Clone for Lock<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl<T: PartialEq + Copy> PartialEq for Lock<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.get() == other.value.get()
    }
}

impl<T: Eq + Copy> Eq for Lock<T> {}

impl<T: PartialOrd + Copy> PartialOrd for Lock<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.get().partial_cmp(&other.value.get())
    }
}

impl<T: Ord + Copy> Ord for Lock<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.get().cmp(&other.value.get())
    }
}
