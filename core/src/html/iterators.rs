//! Layout box iterators

use crate::html::text_format::{FormatSpans, TextSpan};
use crate::string::WStr;
use std::cmp::min;

/// Iterator implementation for the `iter_spans` method of `FormatSpans`.
pub struct TextSpanIter<'a> {
    start_pos: usize,
    base: &'a FormatSpans,
    index: usize,
}

impl<'a> TextSpanIter<'a> {
    pub fn for_format_spans(base: &'a FormatSpans) -> Self {
        Self {
            start_pos: 0,
            base,
            index: 0,
        }
    }
}

impl<'a> Iterator for TextSpanIter<'a> {
    type Item = (usize, usize, &'a WStr, &'a TextSpan);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(span) = self.base.span(self.index) {
            self.index = self.index.saturating_add(1);

            let start_pos = min(self.start_pos, self.base.text().len());
            let end_pos = min(self.start_pos + span.span_length, self.base.text().len());
            let next = (
                start_pos,
                end_pos,
                self.base.displayed_text().slice(start_pos..end_pos)?,
                span,
            );

            self.start_pos = end_pos;

            return Some(next);
        }

        None
    }
}
