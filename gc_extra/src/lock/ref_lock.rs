use core::cell::{BorrowError, Ref, RefCell};
use core::fmt;

use gc_arena::{Collect, CollectionContext};

use super::{Unlock, Write};

/// GC-aware [`RefCell`], only allowing operations compatible with no write barrier.
#[repr(transparent)]
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefLock<T: ?Sized> {
    value: RefCell<T>,
}

impl<T> RefLock<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.value.as_ptr()
    }

    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    #[inline]
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.value.try_borrow()
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
    pub fn unlock(this: &Write<Self>) -> &RefCell<T> {
        &this.value
    }
}

unsafe impl<T: Collect> Collect for RefLock<T> {
    #[inline]
    fn needs_trace() -> bool {
        T::needs_trace()
    }

    #[inline]
    fn trace(&self, cc: CollectionContext) {
        T::trace(&self.value.borrow(), cc)
    }
}

impl<T: ?Sized> Unlock for RefLock<T> {
    type Unlocked = RefCell<T>;

    #[inline]
    unsafe fn unlock_unchecked(&self) -> &RefCell<T> {
        &self.value
    }
}

impl<T> From<T> for RefLock<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for RefLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("RefLock");
        match self.value.try_borrow() {
            Ok(borrow) => f.field("value", &borrow),
            Err(_) => {
                // The RefCell is mutably borrowed so we can't look at its value
                // here. Show a placeholder instead.
                struct BorrowedPlaceholder;

                impl fmt::Debug for BorrowedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }

                f.field("value", &BorrowedPlaceholder)
            }
        }
        .finish()
    }
}
