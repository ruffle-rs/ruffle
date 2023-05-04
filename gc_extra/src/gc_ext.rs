use crate::{lock::Unlock, Write};
use gc_arena::{Gc, MutationContext};

/// Extension trait for `Gc` pointers.
pub trait GcExt<'gc, T: 'gc> {
    /// Triggers a write barrier on the given [`Gc`] pointer, making the referenced value
    /// writable until the next collection.
    fn write(mc: MutationContext<'gc, '_>, this: Self) -> &'gc Write<T>;

    fn unlock(self, mc: MutationContext<'gc, '_>) -> &'gc T::Unlocked
    where
        T: Unlock;
}

impl<'gc, T: 'gc> GcExt<'gc, T> for Gc<'gc, T> {
    #[inline(always)]
    fn write(mc: MutationContext<'gc, '_>, this: Self) -> &'gc Write<T> {
        Gc::write_barrier(mc, this);

        // SAFETY: the write barrier ensures that further mutation is safe, and `'gc` will be
        // invalidated before the next collection (this will be sound once `&'gc T` doesn't
        // implement `Collect`).
        unsafe {
            // Use a raw pointer to extend the lifetime.
            let ptr = Gc::as_ptr(this);
            Write::assume(&*ptr)
        }
    }

    #[inline(always)]
    fn unlock(self, mc: MutationContext<'gc, '_>) -> &'gc T::Unlocked
    where
        T: Unlock,
    {
        GcExt::write(mc, self).unlock()
    }
}
