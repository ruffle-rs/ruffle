//! Provides UCS2 string types for usage in AVM1 and AVM2.
//!
//! Internally, these types are represeted by a sequence of 1-byte or 2-bytes (wide) code units,
//! that may contains null bytes or unpaired surrogates.
//!
//! To match Flash behavior, the string length is limited to 2³¹-1 code units;
//! any attempt to create a longer string will panic.

#[macro_use]
mod common;

mod avm;
mod buf;
mod ops;
mod raw;
mod slice;
mod tables;
pub mod utils;

/// The maximum string length, equals to 2³¹-1.
pub const MAX_STRING_LEN: usize = raw::MAX_STRING_LEN;

pub use avm::AvmString;
pub use buf::WString;
pub use common::{BorrowWStr, BorrowWStrMut, Units};
pub use ops::{Iter, Split};
pub use slice::{WStr, WStrMut};

use common::panic_on_invalid_length;
