//! HTML related utilities

mod dimensions;
mod iterators;
mod layout;
mod text_format;

pub use dimensions::BoxBounds;
pub use dimensions::Position;
pub use dimensions::Size;
pub use layout::{LayoutBox, LayoutContent, LayoutMetrics};
pub use text_format::{FormatSpans, TextFormat, TextSpan};

#[cfg(test)]
mod test;
