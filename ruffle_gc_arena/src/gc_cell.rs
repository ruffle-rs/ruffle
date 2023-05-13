use core::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use core::fmt::{self, Debug, Pointer};

use crate::{Collect, CollectionContext, Gc, GcWeakCell, MutationContext};

/// TODO: replace all usages by `Gc<RefLock<T>>`, `Gc<Lock<T>>`, or similar.
pub struct GcCell<'gc, T: ?Sized + 'gc>(pub(crate) Gc<'gc, GcRefCell<T>>);

impl<'gc, T: ?Sized + 'gc> Copy for GcCell<'gc, T> {}

impl<'gc, T: ?Sized + 'gc> Clone for GcCell<'gc, T> {
    #[inline]
    fn clone(&self) -> GcCell<'gc, T> {
        *self
    }
}

impl<'gc, T: Debug + ?Sized + 'gc> Debug for GcCell<'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.try_read() {
            Ok(borrow) => fmt.debug_struct("GcCell").field("value", &borrow).finish(),
            Err(_) => {
                // The GcCell is mutably borrowed so we can't look at its value
                // here. Show a placeholder instead.
                struct BorrowedPlaceholder;

                impl Debug for BorrowedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }

                fmt.debug_struct("GcCell")
                    .field("value", &BorrowedPlaceholder)
                    .finish()
            }
        }
    }
}

impl<'gc, T: ?Sized + 'gc> Pointer for GcCell<'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), fmt)
    }
}

unsafe impl<'gc, T: ?Sized + 'gc> Collect for GcCell<'gc, T> {
    #[inline]
    fn trace(&self, cc: CollectionContext) {
        self.0.trace(cc)
    }
}

impl<'gc, T: Collect + 'gc> GcCell<'gc, T> {
    #[inline]
    pub fn allocate(mc: MutationContext<'gc, '_>, t: T) -> GcCell<'gc, T> {
        GcCell(Gc::new(
            mc,
            GcRefCell {
                cell: RefCell::new(t),
            },
        ))
    }
}

impl<'gc, T: ?Sized + 'gc> GcCell<'gc, T> {
    #[inline]
    pub fn downgrade(this: GcCell<'gc, T>) -> GcWeakCell<'gc, T> {
        GcWeakCell {
            inner: Gc::downgrade(this.0),
        }
    }

    #[inline]
    pub fn ptr_eq(this: GcCell<'gc, T>, other: GcCell<'gc, T>) -> bool {
        this.as_ptr() == other.as_ptr()
    }

    #[inline]
    pub fn as_ptr(self) -> *mut RefCell<T> {
        Gc::as_ptr(self.0) as *mut _
    }

    #[track_caller]
    #[inline]
    pub fn read(&self) -> Ref<'_, T> {
        self.0.cell.borrow()
    }

    #[inline]
    pub fn try_read(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.cell.try_borrow()
    }

    #[track_caller]
    #[inline]
    pub fn write<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, T> {
        let b = self.0.cell.borrow_mut();
        Gc::write_barrier(mc, self.0);
        b
    }

    #[inline]
    pub fn try_write<'a>(
        &'a self,
        mc: MutationContext<'gc, '_>,
    ) -> Result<RefMut<'a, T>, BorrowMutError> {
        let mb = self.0.cell.try_borrow_mut()?;
        Gc::write_barrier(mc, self.0);
        Ok(mb)
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
        self.0.cell.borrow_mut()
    }

    /// Call `RefCell::try_borrow_mut` on the inner `RefCell` *without* the write barrier.
    ///
    /// # Safety
    /// The safety requirements of this method are exactly the same as [`GcCell::borrow_mut`].
    #[inline]
    pub unsafe fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.cell.try_borrow_mut()
    }
}

#[repr(transparent)]
pub(crate) struct GcRefCell<T: ?Sized> {
    cell: RefCell<T>,
}

unsafe impl<'gc, T: Collect + ?Sized + 'gc> Collect for GcRefCell<T> {
    #[inline]
    fn trace(&self, cc: CollectionContext) {
        self.cell.borrow().trace(cc);
    }
}
