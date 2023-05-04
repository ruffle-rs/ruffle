//! GC-aware interior mutability types, for usage inside GC'd object graphs.

#[allow(clippy::module_inception)]
mod lock;
mod ref_lock;

use crate::barrier::Write;

pub use lock::Lock;
pub use ref_lock::RefLock;

/// Types providing extra operations when behind a [`Write`] reference.
pub trait Unlock {
    /// The unlocked form of the type.
    type Unlocked: ?Sized;

    /// Provides unsafe access to the unlocked form without a write barrier.
    ///
    /// # Safety
    /// This must not be used to violate the garbage collector's invariants;
    /// see [`Write::assume`] for more details.
    unsafe fn unlock_unchecked(&self) -> &Self::Unlocked;
}
