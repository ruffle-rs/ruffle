//! Layout box structure

use crate::context::UpdateContext;
use crate::font::{EvalParameters, Font};
use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cmp::{max, min};
use std::sync::Arc;
use swf::Twips;

/// Contains information relating to the current layout operation.
pub struct LayoutContext<'a, 'gc> {
    /// The movie this layout context is pulling fonts from.
    movie: Arc<SwfMovie>,

    /// The position to put text into.
    ///
    /// This cursor does not take indents, left margins, or alignment into
    /// account. It's X coordinate is always relative to the start of the
    /// current line, not the left edge of the text field being laid out.
    cursor: Position<Twips>,

    /// The resolved font object to use when measuring text.
    font: Option<Font<'gc>>,

    /// The underlying bundle of text being formatted.
    text: &'a str,

    /// The highest font size observed within the current line.
    max_font_size: Twips,

    /// The start of the current chain of layout boxes.
    first_box: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The end of the current chain of layout boxes.
    last_box: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The exterior bounds of all laid-out text, including left and right
    /// margins.
    ///
    /// None indicates that no bounds have yet to be calculated. If the layout
    /// ends without a line having been generated, the default bounding box
    /// should be used.
    exterior_bounds: Option<BoxBounds<Twips>>,

    /// Whether or not we are laying out the first line of a paragraph.
    is_first_line: bool,

    /// Whether or not we encountered a line break of any kind during layout.
    ///
    /// Flash always applies at least one count of the line leading to the
    /// total text bounds of the laid-out text, even if there are no line
    /// breaks to host that leading. This flags that we need to add leading to
    /// the singular line if we have yet to process a newline.
    has_line_break: bool,

    /// All layout boxes in the current line being laid out.
    current_line: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The right margin of the first span in the current line.
    current_line_span: TextSpan,

    /// The total width of the text field being laid out.
    max_bounds: Twips,
}

impl<'a, 'gc> LayoutContext<'a, 'gc> {
    fn new(movie: Arc<SwfMovie>, max_bounds: Twips, text: &'a str) -> Self {
        Self {
            movie,
            cursor: Default::default(),
            font: None,
            text,
            max_font_size: Default::default(),
            first_box: None,
            last_box: None,
            exterior_bounds: None,
            is_first_line: true,
            has_line_break: false,
            current_line: None,
            current_line_span: Default::default(),
            max_bounds,
        }
    }

    /// Calculate the font-provided leading present on this line.
    fn font_leading_adjustment(&self) -> Twips {
        // Flash appears to round up the font's leading to the nearest pixel
        // and adds one. I'm not sure why.
        self.font
            .map(|f| f.get_leading_for_height(self.max_font_size))
            .unwrap_or_else(|| Twips::new(0))
    }

    /// Calculate the line-to-line leading present on ths line, including the
    /// font-leading above.
    fn line_leading_adjustment(&self) -> Twips {
        self.font
            .map(|f| f.get_leading_for_height(self.max_font_size))
            .unwrap_or_else(|| Twips::new(0))
            + Twips::from_pixels(self.current_line_span.leading)
    }

    /// Determine the effective alignment mode for the current line of text.
    ///
    /// This function primarily exists to ensure all bulleted lists are
    /// left-aligned, as no other alignment is respected otherwise.
    fn effective_alignment(&self) -> swf::TextAlign {
        if self.current_line_span.bullet {
            swf::TextAlign::Left
        } else {
            self.current_line_span.align
        }
    }

    /// Apply all indents and alignment to the current line, if necessary.
    ///
    /// The `only_line` parameter should be flagged if this is the only line in
    /// the layout operation (e.g. layout is ending and no newlines have been
    /// encountered yet).
    ///
    /// The `final_line_of_para` parameter should be flagged if this the final
    /// line in the paragraph or layout operation (e.g. it wasn't caused by an
    /// automatic newline and no more text is to be expected).
    fn fixup_line(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        only_line: bool,
        final_line_of_para: bool,
    ) {
        if self.current_line.is_none() {
            return;
        }

        let mut line = self.current_line;
        let mut line_bounds = None;
        let mut box_count: i32 = 0;
        while let Some(linebox) = line {
            let mut write = linebox.write(context.gc_context);
            line = write.next_sibling();
            let (text, _tf, font, params, _color) =
                write.as_renderable_text(self.text).expect("text");

            //Flash ignores trailing spaces when aligning lines, so should we
            if self.current_line_span.align != swf::TextAlign::Left {
                write.bounds = write.bounds.with_size(Size::from(font.measure(
                    text.trim_end(),
                    params,
                    false,
                )));
            }

            if let Some(line_bounds) = &mut line_bounds {
                *line_bounds += write.bounds;
            } else {
                line_bounds = Some(write.bounds);
            }

            box_count += 1;
        }

        let mut line_bounds = line_bounds.unwrap_or_else(Default::default);

        let left_adjustment =
            Self::left_alignment_offset(&self.current_line_span, self.is_first_line);
        let right_adjustment = Twips::from_pixels(self.current_line_span.right_margin);

        let misalignment =
            self.max_bounds - left_adjustment - right_adjustment - line_bounds.width();
        let align_adjustment = max(
            match self.effective_alignment() {
                swf::TextAlign::Left | swf::TextAlign::Justify => Default::default(),
                swf::TextAlign::Center => (misalignment) / 2,
                swf::TextAlign::Right => misalignment,
            },
            Twips::from_pixels(0.0),
        );
        let interim_adjustment = max(
            if !final_line_of_para && self.effective_alignment() == swf::TextAlign::Justify {
                misalignment / max(box_count.saturating_sub(1), 1)
            } else {
                Twips::from_pixels(0.0)
            },
            Twips::from_pixels(0.0),
        );

        let font_leading_adjustment = if only_line {
            self.line_leading_adjustment()
        } else {
            self.font_leading_adjustment()
        };

        if self.current_line_span.bullet {
            self.append_bullet(context, &self.current_line_span.clone());
        }

        line = self.current_line;
        box_count = 0;
        while let Some(linebox) = line {
            let mut write = linebox.write(context.gc_context);

            // TODO: This attempts to keep text of multiple font sizes vertically
            // aligned correctly. It does not consider the baseline of the font,
            // which is information we don't have yet.
            let font_size_adjustment = self.max_font_size - write.bounds.height();

            if write.is_text_box() {
                write.bounds += Position::from((
                    left_adjustment + align_adjustment + (interim_adjustment * box_count),
                    font_size_adjustment + font_leading_adjustment,
                ));
            } else if write.is_bullet() {
                write.bounds += Position::from((
                    Default::default(),
                    font_size_adjustment + font_leading_adjustment,
                ));
            }

            line = write.next_sibling();
            box_count += 1;
        }

        line_bounds +=
            Position::from((left_adjustment + align_adjustment, Twips::from_pixels(0.0)));
        line_bounds += Size::from((Twips::from_pixels(0.0), font_leading_adjustment));

        self.current_line = None;

        if let Some(eb) = &mut self.exterior_bounds {
            *eb += line_bounds;
        } else {
            self.exterior_bounds = Some(line_bounds);
        }
    }

    /// Adjust the text layout cursor down to the next line in response to an
    /// explicit newline.
    ///
    /// This function will also adjust any layout boxes on the current line to
    /// their correct alignment and indentation.
    fn explicit_newline(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.fixup_line(context, false, true);

        self.cursor.set_x(Twips::from_pixels(0.0));
        self.cursor += (
            Twips::from_pixels(0.0),
            self.max_font_size + self.line_leading_adjustment(),
        )
            .into();

        self.is_first_line = true;
        self.has_line_break = true;
    }

    /// Adjust the text layout cursor down to the next line.
    ///
    /// This function will also adjust any layout boxes on the current line to
    /// their correct alignment and indentation.
    fn newline(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.fixup_line(context, false, false);

        self.cursor.set_x(Twips::from_pixels(0.0));
        self.cursor += (
            Twips::from_pixels(0.0),
            self.max_font_size + self.line_leading_adjustment(),
        )
            .into();

        self.is_first_line = false;
        self.has_line_break = true;
    }

    /// Adjust the text layout cursor in response to a tab.
    ///
    /// Tabs can do two separate things in Flash, depending on whether or not
    /// tab stops have been manually determined. If they have been, then the
    /// text cursor goes to the next closest tab stop that has not yet been
    /// passed, or if no such stop exists, tabs do nothing. If no tab stops
    /// exist, then the cursor is advanced to some position modulo the natural
    /// tab index.
    fn tab(&mut self) {
        if self.current_line_span.tab_stops.is_empty() {
            let modulo_factor = Twips::from_pixels(self.current_line_span.size * 2.7);
            let stop_modulo_tab =
                ((self.cursor.x().get() / modulo_factor.get()) + 1) * modulo_factor.get();
            self.cursor.set_x(Twips::new(stop_modulo_tab));
        } else {
            for stop in self.current_line_span.tab_stops.iter() {
                let twips_stop = Twips::from_pixels(*stop);
                if twips_stop > self.cursor.x() {
                    self.cursor.set_x(twips_stop);
                    break;
                }
            }
        }
    }

    /// Enter a new span.
    fn newspan(&mut self, first_span: &TextSpan) {
        if self.current_line.is_none() {
            self.current_line_span = first_span.clone();
        }

        self.max_font_size = max(self.max_font_size, Twips::from_pixels(first_span.size));
    }

    fn resolve_font(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        span: &TextSpan,
    ) -> Option<Font<'gc>> {
        let library = context.library.library_for_movie_mut(self.movie.clone());

        if let Some(font) = library
            .get_font_by_name(&span.font, span.bold, span.italic)
            .or_else(|| library.device_font())
        {
            self.font = Some(font);
            return self.font;
        }

        None
    }

    /// Append text to the current line of the ongoing layout operation.
    ///
    /// The text given may or may not be separated into fragments, depending on
    /// what the layout calls for.
    fn append_text(
        &mut self,
        mc: MutationContext<'gc, '_>,
        text: &'a str,
        start: usize,
        end: usize,
        span: &TextSpan,
    ) {
        if self.effective_alignment() == swf::TextAlign::Justify {
            for word in text.split(' ') {
                let word_start = word.as_ptr() as usize - text.as_ptr() as usize;
                let word_end = min(word_start + word.len() + 1, text.len());

                self.append_text_fragment(
                    mc,
                    text.get(word_start..word_end).unwrap(),
                    start + word_start,
                    start + word_end,
                    span,
                );
            }
        } else {
            self.append_text_fragment(mc, text, start, end, span);
        }
    }

    /// Append text fragments to the current line of the given layout context.
    ///
    /// This function bypasses the text fragmentation necessary for justify to
    /// work and it should only be called internally.
    fn append_text_fragment(
        &mut self,
        mc: MutationContext<'gc, '_>,
        text: &'a str,
        start: usize,
        end: usize,
        span: &TextSpan,
    ) {
        let params = EvalParameters::from_span(span);
        let text_size = Size::from(self.font.unwrap().measure(text, params, false));
        let text_bounds = BoxBounds::from_position_and_size(self.cursor, text_size);
        let new_text = LayoutBox::from_text(mc, start, end, self.font.unwrap(), span);
        let mut write = new_text.write(mc);

        write.bounds = text_bounds;

        self.cursor += Position::from((text_size.width(), Twips::default()));
        self.append_box(mc, new_text);
    }

    /// Append a bullet to the start of the current line.
    ///
    /// The bullet will always be placed at the start of the current line. It
    /// should be appended after line fixup has completed, but before the text
    /// cursor is moved down.
    fn append_bullet(&mut self, context: &mut UpdateContext<'_, 'gc, '_>, span: &TextSpan) {
        let library = context.library.library_for_movie_mut(self.movie.clone());

        if let Some(bullet_font) = library
            .get_font_by_name(&span.font, span.bold, span.italic)
            .or_else(|| library.device_font())
            .or(self.font)
        {
            let mut bullet_cursor = self.cursor;

            bullet_cursor.set_x(Twips::from_pixels(18.0));

            let params = EvalParameters::from_span(span);
            let text_size = Size::from(bullet_font.measure("\u{2022}", params, false));
            let text_bounds = BoxBounds::from_position_and_size(bullet_cursor, text_size);
            let new_bullet = LayoutBox::from_bullet(context.gc_context, bullet_font, span);
            let mut write = new_bullet.write(context.gc_context);

            write.bounds = text_bounds;

            self.append_box(context.gc_context, new_bullet);
        }
    }

    /// Add a box to the current line of text.
    ///
    /// The box should have been positioned according to the current cursor
    /// position. It will be adjusted some time later to properly position it
    /// within the current layout box.
    fn append_box(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        to_append: GcCell<'gc, LayoutBox<'gc>>,
    ) {
        if self.first_box.is_none() {
            self.first_box = Some(to_append);
        }

        if self.current_line.is_none() {
            self.current_line = Some(to_append);
        }

        if let Some(last) = self.last_box {
            last.write(gc_context).next_sibling = Some(to_append);
        }

        self.last_box = Some(to_append);
    }

    /// Calculate the left-align offset of a given line of text given the span
    /// active at the start of the line and if we're at the start of a
    /// paragraph.
    fn left_alignment_offset(span: &TextSpan, is_first_line: bool) -> Twips {
        if span.bullet {
            if is_first_line {
                Twips::from_pixels(35.0 + span.left_margin + span.block_indent + span.indent)
            } else {
                Twips::from_pixels(35.0 + span.left_margin + span.block_indent)
            }
        } else if is_first_line {
            Twips::from_pixels(span.left_margin + span.block_indent + span.indent)
        } else {
            Twips::from_pixels(span.left_margin + span.block_indent)
        }
    }

    /// Calculate the left and right bounds of a wrapping operation based on
    /// the current state of the layout operation.
    ///
    /// This function yields both a remaining line width and an offset into
    /// that line. Those should be passed as the `width` and `offset`
    /// parameters of `Font.wrap_line`.
    ///
    /// Offsets returned by this function should not be considered final;
    fn wrap_dimensions(&self, current_span: &TextSpan) -> (Twips, Twips) {
        let width = self.max_bounds - Twips::from_pixels(self.current_line_span.right_margin);
        let offset = Self::left_alignment_offset(current_span, self.is_first_line);

        (width, offset + self.cursor.x())
    }

    /// Destroy the layout context, returning the newly constructed layout list.
    fn end_layout(
        mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> (Option<GcCell<'gc, LayoutBox<'gc>>>, BoxBounds<Twips>) {
        self.fixup_line(context, !self.has_line_break, true);

        (
            self.first_box,
            self.exterior_bounds.unwrap_or_else(Default::default),
        )
    }

    fn is_start_of_line(&self) -> bool {
        self.current_line.is_none()
    }
}

/// A `LayoutBox` represents a series of nested content boxes, each of which
/// may contain a single line of text with a given text format applied to it.
///
/// Layout boxes are nested and can optionally be associated with an HTML
/// element. The relationship between elements and boxes are nullably
/// one-to-many: an HTML element may be represented by multiple layout boxes,
/// while a layout may have zero or one HTML elements attached to it. This
/// allows inline content
///
/// They also have margins, padding, and borders which are calculated and
/// rendered according to CSS spec.
///
/// For example, an HTML tree that looks like this:
///
/// ```<p>I'm a <i>layout element</i> node!</p>```
///
/// produces a top-level `LayoutBox` for the `<p>` tag, which contains one or
/// more generated boxes for each run of text. The `<i>` tag is cut at it's
/// whitespace if necessary and the cut pieces are added to their respective
/// generated boxes.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LayoutBox<'gc> {
    /// The rectangle corresponding to the outer boundaries of the
    bounds: BoxBounds<Twips>,

    /// The layout box to be placed after this one.
    next_sibling: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// What content is contained by the content box.
    content: LayoutContent<'gc>,
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct Collec<T>(T);

/// Represents different content modes of a given layout box.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum LayoutContent<'gc> {
    /// A layout box containing some part of a text span.
    ///
    /// The text is assumed to be pulled from the same `FormatSpans` that
    /// generated this layout box.
    Text {
        /// The start position of the text to render.
        start: usize,

        /// The end position of the text to render.
        end: usize,

        /// The formatting options for the text box.
        text_format: TextFormat,

        /// The font that was resolved at the time of layout for this text.
        font: Font<'gc>,

        /// All parameters used to evaluate the font at the time of layout for
        /// this text.
        params: EvalParameters,

        /// The color to render the font with.
        color: Collec<swf::Color>,
    },

    /// A layout box containing a bullet.
    ///
    /// This is almost identical to `Text`, but the text contents are assumed
    /// to be U+2022.
    Bullet {
        /// The formatting options for the text box.
        text_format: TextFormat,

        /// The font that was resolved at the time of layout for this text.
        font: Font<'gc>,

        /// All parameters used to evaluate the font at the time of layout for
        /// this text.
        params: EvalParameters,

        /// The color to render the font with.
        color: Collec<swf::Color>,
    },
}

impl<'gc> LayoutBox<'gc> {
    /// Construct a text box for a text node.
    pub fn from_text(
        mc: MutationContext<'gc, '_>,
        start: usize,
        end: usize,
        font: Font<'gc>,
        span: &TextSpan,
    ) -> GcCell<'gc, Self> {
        let params = EvalParameters::from_span(span);

        GcCell::allocate(
            mc,
            Self {
                bounds: Default::default(),
                next_sibling: None,
                content: LayoutContent::Text {
                    start,
                    end,
                    text_format: span.get_text_format(),
                    font,
                    params,
                    color: Collec(span.color.clone()),
                },
            },
        )
    }

    /// Construct a bullet.
    pub fn from_bullet(
        mc: MutationContext<'gc, '_>,
        font: Font<'gc>,
        span: &TextSpan,
    ) -> GcCell<'gc, Self> {
        let params = EvalParameters::from_span(span);

        GcCell::allocate(
            mc,
            Self {
                bounds: Default::default(),
                next_sibling: None,
                content: LayoutContent::Bullet {
                    text_format: span.get_text_format(),
                    font,
                    params,
                    color: Collec(span.color.clone()),
                },
            },
        )
    }

    /// Returns the next sibling box.
    pub fn next_sibling(&self) -> Option<GcCell<'gc, LayoutBox<'gc>>> {
        self.next_sibling
    }

    /// Construct a new layout hierarchy from text spans.
    ///
    /// The returned bounds will include both the text bounds itself, as well
    /// as left and right margins on any of the lines.
    pub fn lower_from_text_spans(
        fs: &FormatSpans,
        context: &mut UpdateContext<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        bounds: Twips,
        is_word_wrap: bool,
    ) -> (Option<GcCell<'gc, LayoutBox<'gc>>>, BoxBounds<Twips>) {
        let mut layout_context = LayoutContext::new(movie, bounds, fs.text());

        for (span_start, _end, span_text, span) in fs.iter_spans() {
            if let Some(font) = layout_context.resolve_font(context, &span) {
                layout_context.newspan(span);

                let params = EvalParameters::from_span(span);

                for text in span_text.split(&['\n', '\t'][..]) {
                    let slice_start = text.as_ptr() as usize - span_text.as_ptr() as usize;
                    let delimiter = if slice_start > 0 {
                        span_text
                            .get(slice_start - 1..)
                            .and_then(|s| s.chars().next())
                    } else {
                        None
                    };

                    match delimiter {
                        Some('\n') => layout_context.explicit_newline(context),
                        Some('\t') => layout_context.tab(),
                        _ => {}
                    }

                    let start = span_start + slice_start;

                    let mut last_breakpoint = 0;

                    if is_word_wrap {
                        let (mut width, mut offset) = layout_context.wrap_dimensions(&span);

                        while let Some(breakpoint) = font.wrap_line(
                            &text[last_breakpoint..],
                            params,
                            width,
                            offset,
                            layout_context.is_start_of_line(),
                        ) {
                            if breakpoint == 0 {
                                layout_context.newline(context);

                                let next_dim = layout_context.wrap_dimensions(&span);

                                width = next_dim.0;
                                offset = next_dim.1;

                                if last_breakpoint >= text.len() {
                                    break;
                                } else {
                                    continue;
                                }
                            }

                            // This ensures that the space causing the line break
                            // is included in the line it broke.
                            let next_breakpoint = min(last_breakpoint + breakpoint + 1, text.len());

                            layout_context.append_text(
                                context.gc_context,
                                &text[last_breakpoint..next_breakpoint],
                                start + last_breakpoint,
                                start + next_breakpoint,
                                span,
                            );

                            last_breakpoint = next_breakpoint;
                            if last_breakpoint >= text.len() {
                                break;
                            }

                            layout_context.newline(context);
                            let next_dim = layout_context.wrap_dimensions(&span);

                            width = next_dim.0;
                            offset = next_dim.1;
                        }
                    }

                    let span_end = text.len();

                    if last_breakpoint < span_end {
                        layout_context.append_text(
                            context.gc_context,
                            &text[last_breakpoint..span_end],
                            start + last_breakpoint,
                            start + span_end,
                            span,
                        );
                    }
                }
            }
        }

        layout_context.end_layout(context)
    }

    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    /// Returns a reference to the text this box contains.
    pub fn as_renderable_text<'a>(
        &self,
        text: &'a str,
    ) -> Option<(&'a str, &TextFormat, Font<'gc>, EvalParameters, swf::Color)> {
        match &self.content {
            LayoutContent::Text {
                start,
                end,
                text_format,
                font,
                params,
                color,
            } => Some((
                text.get(*start..*end)?,
                &text_format,
                *font,
                *params,
                color.0.clone(),
            )),
            LayoutContent::Bullet {
                text_format,
                font,
                params,
                color,
            } => Some(("\u{2022}", &text_format, *font, *params, color.0.clone())),
        }
    }

    pub fn is_text_box(&self) -> bool {
        match &self.content {
            LayoutContent::Text { .. } => true,
            LayoutContent::Bullet { .. } => false,
        }
    }

    pub fn is_bullet(&self) -> bool {
        match &self.content {
            LayoutContent::Text { .. } => false,
            LayoutContent::Bullet { .. } => true,
        }
    }

    /// Construct a duplicate layout box structure.
    pub fn duplicate(&self, context: MutationContext<'gc, '_>) -> GcCell<'gc, Self> {
        GcCell::allocate(
            context,
            Self {
                bounds: self.bounds,
                next_sibling: self.next_sibling.map(|ns| ns.read().duplicate(context)),
                content: self.content.clone(),
            },
        )
    }
}
