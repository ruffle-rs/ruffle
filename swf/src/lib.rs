//! # swf-rs
//!
//! Library for reading and writing Adobe Flash SWF files.
//!
//! # Organization
//!
//! This library consits of a `read` module for decoding SWF data, and a `write` library for
//! writing SWF data.

extern crate byteorder;
#[cfg(feature = "flate2")]
extern crate flate2;
#[cfg(feature = "libflate")]
extern crate libflate;
#[macro_use]
extern crate num_derive;
extern crate num_traits;
#[cfg(feature = "lzma-support")]
extern crate xz2;

pub mod avm1;
pub mod avm2;
pub mod error;
pub mod read;
mod tag_code;
mod types;
pub mod write;

#[cfg(test)]
mod test_data;

/// Reexports
pub use read::{read_swf, read_swf_header};
pub use tag_code::TagCode;
pub use types::*;
pub use write::write_swf;
