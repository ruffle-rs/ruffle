use core::cell::{BorrowError, BorrowMutError, Ref, RefMut};
use core::fmt::{self, Debug, Pointer};

use crate::lock::RefLock;
use crate::{Collect, Collection, Gc, GcWeakCell, Mutation};

/// TODO: replace all usages by `Gc<RefLock<T>>`, `Gc<Lock<T>>`, or similar.
pub struct GcCell<'gc, T: ?Sized + 'gc>(pub(crate) Gc<'gc, RefLock<T>>);

impl<'gc, T: ?Sized + 'gc> Copy for GcCell<'gc, T> {}

impl<'gc, T: ?Sized + 'gc> Clone for GcCell<'gc, T> {
    #[inline]
    fn clone(&self) -> GcCell<'gc, T> {
        *self
    }
}

impl<'gc, T: Debug + ?Sized + 'gc> Debug for GcCell<'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, fmt)
    }
}

impl<'gc, T: ?Sized + 'gc> Pointer for GcCell<'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), fmt)
    }
}

unsafe impl<'gc, T: ?Sized + 'gc> Collect for GcCell<'gc, T> {
    #[inline]
    fn trace(&self, cc: &Collection) {
        self.0.trace(cc)
    }
}

impl<'gc, T: Collect + 'gc> GcCell<'gc, T> {
    #[inline]
    pub fn new(mc: &Mutation<'gc>, t: T) -> GcCell<'gc, T> {
        GcCell(Gc::new(mc, RefLock::new(t)))
    }
}

impl<'gc, T: ?Sized + 'gc> GcCell<'gc, T> {
    #[inline]
    pub fn downgrade(this: GcCell<'gc, T>) -> GcWeakCell<'gc, T> {
        GcWeakCell(Gc::downgrade(this.0))
    }

    #[inline]
    pub fn ptr_eq(this: GcCell<'gc, T>, other: GcCell<'gc, T>) -> bool {
        Gc::ptr_eq(this.0, other.0)
    }

    #[inline]
    pub fn as_ptr(self) -> *const RefLock<T> {
        Gc::as_ptr(self.0)
    }

    #[track_caller]
    #[inline]
    pub fn read(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    #[inline]
    pub fn try_read(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.try_borrow()
    }

    #[track_caller]
    #[inline]
    pub fn write<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, T> {
        self.0.borrow_mut(mc)
    }

    #[inline]
    pub fn try_write<'a>(&'a self, mc: &Mutation<'gc>) -> Result<RefMut<'a, T>, BorrowMutError> {
        self.0.try_borrow_mut(mc)
    }

    /// Call `RefCell::borrow_mut` on the inner `RefCell` *without* the write barrier.
    ///
    /// # Safety
    /// In order to maintain the invariants of the garbage collector, no new `Gc` pointers
    /// may be adopted by this type as a result of the interior mutability afforded here, unless the
    /// write barrier is invoked manually before collection is triggered.
    #[track_caller]
    #[inline]
    pub unsafe fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.as_ref_cell().borrow_mut()
    }

    /// Call `RefCell::try_borrow_mut` on the inner `RefCell` *without* the write barrier.
    ///
    /// # Safety
    /// The safety requirements of this method are exactly the same as [`GcCell::borrow_mut`].
    #[inline]
    pub unsafe fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.as_ref_cell().try_borrow_mut()
    }
}
