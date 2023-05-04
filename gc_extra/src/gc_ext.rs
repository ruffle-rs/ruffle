use crate::Write;
use gc_arena::{Gc, MutationContext};

/// Extension trait for `Gc` pointers.
pub trait GcExt<'gc, T> {
    /// Triggers a write barrier on the given [`Gc`] pointer, making the referenced value
    /// writable until the next collection.
    fn write<'a>(mc: MutationContext<'gc, 'a>, this: Self) -> &'a Write<T>;
}

impl<'gc, T> GcExt<'gc, T> for Gc<'gc, T> {
    #[inline(always)]
    fn write<'a>(mc: MutationContext<'gc, 'a>, this: Self) -> &'a Write<T> {
        Gc::write_barrier(mc, this);

        // SAFETY: the write barrier ensures that further mutation is safe, and `'a` will be
        // invalidated before the next collection (as a `&'gc Gc<'gc, _>` cannot soundly exist).
        unsafe {
            // Use a raw pointer to extend the lifetime.
            let ptr = Gc::as_ptr(this);
            Write::assume(&*ptr)
        }
    }
}
