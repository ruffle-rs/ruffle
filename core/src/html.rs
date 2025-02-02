//! HTML related utilities

mod dimensions;
mod iterators;
mod layout;
mod text_format;

pub use dimensions::Position;
pub use layout::{
    lower_from_text_spans, Layout, LayoutBox, LayoutContent, LayoutLine, LayoutMetrics,
};
pub use style_sheet::{parse_font_list, transform_dashes_to_camel_case, CssStream, StyleSheet};
pub use text_format::{FormatSpans, TextDisplay, TextFormat, TextSpan};

mod style_sheet;
#[cfg(test)]
mod test;
