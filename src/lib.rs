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
pub mod read;
mod tag_codes;
mod types;
pub mod write;

#[cfg(test)]
mod test_data;

/// Parses an SWF from a `Read` stream.
pub use read::{read_swf, read_swf_header};

/// Writes an SWF to a `Write` stream.
pub use write::write_swf;

/// Types used to represent a parsed SWF.
pub use types::*;
