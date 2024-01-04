use crate::lock::RefLock;
use crate::{Collect, Collection, GcCell, GcWeak, Mutation};

use core::fmt::{self, Debug};

/// TODO: replace all usages by `GcWeak<RefLock<T>>`, `GcWeak<Lock<T>>`, or similar.
pub struct GcWeakCell<'gc, T: ?Sized + 'gc>(pub(crate) GcWeak<'gc, RefLock<T>>);

impl<'gc, T: ?Sized + 'gc> Copy for GcWeakCell<'gc, T> {}

impl<'gc, T: ?Sized + 'gc> Clone for GcWeakCell<'gc, T> {
    #[inline]
    fn clone(&self) -> GcWeakCell<'gc, T> {
        *self
    }
}

impl<'gc, T: ?Sized + 'gc> Debug for GcWeakCell<'gc, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, fmt)
    }
}

unsafe impl<'gc, T: ?Sized + 'gc> Collect for GcWeakCell<'gc, T> {
    #[inline]
    fn trace(&self, cc: &Collection) {
        self.0.trace(cc);
    }
}

impl<'gc, T: ?Sized + 'gc> GcWeakCell<'gc, T> {
    #[inline]
    pub fn upgrade(&self, mc: &Mutation<'gc>) -> Option<GcCell<'gc, T>> {
        self.0.upgrade(mc).map(GcCell)
    }

    #[inline]
    pub fn ptr_eq(this: GcWeakCell<'gc, T>, other: GcWeakCell<'gc, T>) -> bool {
        GcWeak::ptr_eq(this.0, other.0)
    }

    #[inline]
    pub fn as_ptr(self) -> *const RefLock<T> {
        self.0.as_ptr()
    }
}
