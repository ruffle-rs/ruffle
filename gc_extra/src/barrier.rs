use core::ops::{Deref, DerefMut};

use crate::lock::Unlock;

/// An (interiorly-)mutable reference inside a GC'd object graph.
///
/// TODO: explain in more detail how to use this.
#[repr(transparent)]
#[non_exhaustive]
pub struct Write<T: ?Sized> {
    // Public so that the projection macros can access it.
    #[doc(hidden)]
    pub inner: T,
}

impl<T: ?Sized> Deref for Write<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for Write<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> Write<T> {
    /// Asserts that the given reference can be safely written to.
    ///
    /// # Safety
    /// In order to maintain the invariants of the garbage collector, no new [`Gc`] pointers
    /// may be adopted by the referenced value as a result of the interior mutability enabled
    /// by this wrapper, unless [`Gc::write_barrier`] is invoked manually on the parent [`Gc`]
    /// pointer before the next collection is triggered.
    ///
    /// Note that write barriers (and, consequently, `Write` references) do not propagate
    /// to nested [`Gc`] pointers.
    /// ```
    /// # use gc_arena::{Gc, MutationContext};
    /// # use ruffle_gc_extra::{lock::Lock, Write};
    /// // This is unsound!
    /// fn set_nested<T>(dst: &Write<Gc<'_, Lock<T>>>, value: T) {
    ///     // WRONG! writeability doesn't apply to the `Lock<T>`.
    ///     let nested = unsafe { Write::<Lock<_>>::assume(&*dst) };
    ///     // There was no write barrier on the inner `Gc` pointer, so any `Gc` pointers held
    ///     // by `value` may be mistaken as garbage and dropped prematurely in future collections.
    ///     nested.unlock().set(value);
    /// }
    /// ```
    #[inline(always)]
    pub unsafe fn assume(v: &T) -> &Self {
        // SAFETY: Self is transparent over T
        core::mem::transmute(v)
    }

    /// Gets a writable reference from a `&mut T`.
    ///
    /// This is safe, as exclusive access already implies writability.
    #[inline(always)]
    pub fn from_mut(v: &mut T) -> &mut Self {
        // SAFETY: Self is transparent over T
        unsafe { core::mem::transmute(v) }
    }

    /// Gets a writable reference to non-GC'd data.
    ///
    /// This is safe, as `'static` types can never hold (unleaked) [`Gc`] pointers.
    #[inline(always)]
    pub fn from_static(v: &T) -> &Self
    where
        T: 'static,
    {
        // SAFETY: Self is transparent over T
        unsafe { core::mem::transmute(v) }
    }

    /// Implementation detail of `write_field!`; same safety requirements as `assume`.
    #[inline(always)]
    #[doc(hidden)]
    pub unsafe fn __from_ref_and_ptr(v: &T, _: *const T) -> &Self {
        // SAFETY: Self is transparent over T
        unsafe { core::mem::transmute(v) }
    }

    /// Unlocks the referenced value, providing full interior mutability.
    ///
    /// # Note
    /// This method (and the `Unlock` trait) mostly exists to
    /// work-around the lack of `arbitrary_self_types` on stable Rust.
    #[inline(always)]
    pub fn unlock(&self) -> &T::Unlocked
    where
        T: Unlock,
    {
        // SAFETY: `Write` asserts that the value can be soundly unlocked.
        unsafe { self.inner.unlock_unchecked() }
    }
}

/// Named field projection behind [`Write`] references.
///
/// TODO: document usages
#[macro_export]
macro_rules! field {
    ($value:expr, $type:path, $field:ident) => {{
        // SAFETY:
        // For this to be sound, we need to prevent deref coercions from happening, as they may
        // access nested `Gc` pointers, which would violate the write barrier invariant. This is
        // guaranteed as follows:
        // - the destructuring pattern, unlike a simple field access, cannot call `Deref`;
        // - similarly, the `__from_ref_and_ptr` method takes both a reference (for the lifetime)
        //   and a pointer, causing a compilation failure if the first argument was coerced.
        match $value {
            $crate::Write {
                inner: $type { ref $field, .. },
                ..
            } => unsafe { $crate::Write::__from_ref_and_ptr($field, $field as *const _) },
        }
    }};
}

/// Shorthand for [`write_field!`]`(...).`[`unlock()`](Write::unlock).
#[macro_export]
macro_rules! unlock {
    ($value:expr, $type:path, $field:ident) => {
        $crate::Write::unlock($crate::field!($value, $type, $field))
    };
}
