use gc_arena::{Collect, Gc};

pub use ruffle_macros::HasPrefixField;

/// A trait indicating that `Self` has `Inner` as an initial prefix.
///
/// A field prefix is the first field in a struct that has the same address as the struct
/// in the memory. If a struct has a prefix field, we can reinterpret the struct pointer
/// as a pointer to the field.
///
/// Implementing this trait provides various methods to cast `Self` references to `Inner`
/// references, which can be used e.g. to implement OOP-style class hierarchies.
///
/// This trait can be automatically derived on `repr(C)` structs with `#[derive(HasPrefixField)]`.
///
/// # Safety
/// - `Self` must have a field of type `Inner` at the start of its layout;
/// - `Self` must not impose additional safety invariants on the `Inner` prefix;
/// - The methods of this trait should not be overriden;
/// - Any layout constraints that can't be checked by the type-system should
///   be checked by assertions in the `ASSERT_PREFIX_FIELD` constant.
pub unsafe trait HasPrefixField<Inner>: Sized {
    /// This constant should *always* be evaluated before relying on the safety
    /// guarantees of this trait.
    const ASSERT_PREFIX_FIELD: () = ();

    /// Casts a GC'd object to its prefix field.
    #[inline(always)]
    fn as_prefix_gc(gc: Gc<'_, Self>) -> Gc<'_, Inner> {
        // Casting between `Gc`s currently requires matching alignment.
        const { assert!(align_of::<Self>() == align_of::<Inner>()) };
        let () = Self::ASSERT_PREFIX_FIELD;
        // SAFETY: The above asserts guarantee that the layouts are compatible.
        unsafe { Gc::cast(gc) }
    }
}

/// A `u8` which is always zero. Useful to artificially introduce niches into a struct.
#[derive(Copy, Clone, Collect, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[collect(require_static)]
#[repr(u8)]
pub enum ZeroU8 {
    #[default]
    Zero = 0,
}
