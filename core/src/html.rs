//! HTML related utilities

mod dimensions;
mod iterators;
mod layout;
mod text_format;

pub use text_format::{FormatSpans, TextFormat};

#[cfg(test)]
mod test;
