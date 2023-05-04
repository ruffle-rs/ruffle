#![no_std]

mod barrier;
mod gc_ext;
pub mod lock;

pub use barrier::Write;
pub use gc_ext::GcExt;

use gc_arena::Gc;

pub type GcLock<'gc, T> = Gc<'gc, lock::Lock<T>>;
pub type GcRefLock<'gc, T> = Gc<'gc, lock::RefLock<T>>;
