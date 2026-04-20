//! Provides the string implementations used in AVM1 and AVM2.
//!
//! This uses the same representation as `wstr`, but also includes support for
//! garbage collection, dependent strings, and interning.
#![expect(clippy::module_inception)]

mod avm_string;
mod common;
mod context;
mod interner;
mod repr;

pub use avm_string::AvmString;
pub use common::CommonStrings;
pub use context::{HasStringContext, StringContext};
pub use interner::{AvmAtom, AvmStringInterner};
