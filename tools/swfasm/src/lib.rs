//! A textual (RON) representation of SWF files, with a compiler that turns it
//! into SWF files.

mod compile;
pub mod source;

pub use compile::{compile, compile_str, parse};
