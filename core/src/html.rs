//! HTML related utilities

mod dimensions;
mod iterators;
mod layout;
mod text_format;

pub use dimensions::Position;
pub use layout::{
    Layout, LayoutBox, LayoutContent, LayoutLine, LayoutMetrics, lower_from_text_spans,
};
pub use style_sheet::{CssStream, StyleSheet, parse_font_list, transform_dashes_to_camel_case};
pub use text_format::{FormatSpans, TextDisplay, TextFormat, TextSpan};

mod style_sheet;
#[cfg(test)]
mod test;
