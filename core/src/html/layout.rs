//! Layout box structure

use crate::context::UpdateContext;
use crate::drawing::Drawing;
use crate::font::{EvalParameters, Font};
use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::string::{utils as string_utils, WStr};
use crate::tag_utils::SwfMovie;
use gc_arena::Collect;
use ruffle_render::shape_utils::DrawCommand;
use std::cmp::{max, min};
use std::sync::Arc;
use crate::prelude::Twips;

/// Draw an underline on a particular drawing.
///
/// This will not draw underlines shorter than a pixel in width.
fn draw_underline(drawing: &mut Drawing, starting_pos: Position<Twips>, width: Twips) {
    if width < Twips::from_pixels(1.0) {
        return;
    }

    let ending_pos = starting_pos + Position::from((width, Twips::ZERO));

    drawing.draw_command(DrawCommand::MoveTo {
        x: starting_pos.x(),
        y: starting_pos.y(),
    });
    drawing.draw_command(DrawCommand::LineTo {
        x: ending_pos.x(),
        y: ending_pos.y(),
    });
}

/// Contains information relating to the current layout operation.
pub struct LayoutContext<'a, 'gc> {
    /// The movie this layout context is pulling fonts from.
    movie: Arc<SwfMovie>,

    /// The position to put text into.
    ///
    /// This cursor does not take indents, left margins, or alignment into
    /// account. Its X coordinate is always relative to the start of the
    /// current line, not the left edge of the text field being laid out.
    cursor: Position<Twips>,

    /// The resolved font object to use when measuring text.
    font: Option<Font<'gc>>,

    /// The underlying bundle of text being formatted.
    text: &'a WStr,

    /// The highest font size observed within the current line.
    max_font_size: Twips,

    /// The growing list of layout boxes to return when layout has finished.
    boxes: Vec<LayoutBox<'gc>>,

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

    /// The first box within the current line.
    ///
    /// If equal to the length of the array, then no layout boxes currently
    /// exist for this line.
    current_line: usize,

    /// The right margin of the first span in the current line.
    current_line_span: TextSpan,

    /// The total width of the text field being laid out.
    max_bounds: Twips,
}

impl<'a, 'gc> LayoutContext<'a, 'gc> {
    fn new(movie: Arc<SwfMovie>, max_bounds: Twips, text: &'a WStr) -> Self {
        Self {
            movie,
            cursor: Default::default(),
            font: None,
            text,
            max_font_size: Default::default(),
            boxes: Vec::new(),
            exterior_bounds: None,
            is_first_line: true,
            has_line_break: false,
            current_line: 0,
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
            .unwrap_or_default()
    }

    /// Calculate the line-to-line leading present on this line, including the
    /// font-leading above.
    fn line_leading_adjustment(&self) -> Twips {
        self.font
            .map(|f| f.get_leading_for_height(self.max_font_size))
            .unwrap_or_default()
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

    /// Construct an underline drawing for the current line of text and add it
    /// to the line.
    fn append_underlines(&mut self) {
        let mut starting_pos: Option<Position<Twips>> = None;
        let mut current_width: Option<Twips> = None;
        let mut line_drawing = Drawing::new();
        let mut has_underline: bool = false;

        line_drawing.set_line_style(Some(
            swf::LineStyle::new()
                .with_width(Twips::new(1))
                .with_color(swf::Color::BLACK),
        ));

        if let Some(linelist) = self.boxes.get(self.current_line..) {
            for linebox in linelist {
                if linebox.is_text_box() {
                    if let Some((_t, tf, font, params, _color)) =
                        linebox.as_renderable_text(self.text)
                    {
                        let underline_baseline =
                            font.get_baseline_for_height(params.height()) + Twips::from_pixels(2.0);
                        let mut line_extended = false;

                        if let Some(starting_pos) = starting_pos {
                            if tf.underline.unwrap_or(false)
                                && underline_baseline + linebox.bounds().origin().y()
                                    == starting_pos.y()
                            {
                                //Underline is at the same baseline, extend it
                                current_width =
                                    Some(linebox.bounds().extent_x() - starting_pos.x());

                                line_extended = true;
                            }
                        }

                        if !line_extended {
                            //For whatever reason, we cannot extend the current underline.
                            //This can happen if we don't have an underline to extend, the
                            //underlines don't match, or this span doesn't call for one.
                            if let (Some(pos), Some(width)) = (starting_pos, current_width) {
                                draw_underline(&mut line_drawing, pos, width);
                                has_underline = true;
                                starting_pos = None;
                                current_width = None;
                            }

                            if tf.underline.unwrap_or(false) {
                                starting_pos = Some(
                                    linebox.bounds().origin()
                                        + Position::from((Twips::ZERO, underline_baseline)),
                                );
                                current_width = Some(linebox.bounds().width());
                            }
                        }
                    }
                }
            }
        }

        if let (Some(starting_pos), Some(current_width)) = (starting_pos, current_width) {
            draw_underline(&mut line_drawing, starting_pos, current_width);
            has_underline = true;
        }

        if has_underline {
            self.append_box(LayoutBox::from_drawing(line_drawing));
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
    ///
    /// The `text` parameter, if provided, consists of the master text slice,
    /// the current read index into that slice, and the current text span we
    /// are formatting. This is for empty line insertion; omitting this
    /// parameter will result in no empty lines being added.
    fn fixup_line(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        only_line: bool,
        final_line_of_para: bool,
        text: Option<(&'a WStr, usize, &TextSpan)>,
    ) {
        if self.boxes.get_mut(self.current_line..).is_none() {
            return;
        }

        let mut line_bounds = None;
        let mut box_count: i32 = 0;
        for linebox in self.boxes.get_mut(self.current_line..).unwrap() {
            let (text, _tf, font, params, _color) =
                linebox.as_renderable_text(self.text).expect("text");

            //Flash ignores trailing spaces when aligning lines, so should we
            if self.current_line_span.align != swf::TextAlign::Left {
                linebox.bounds = linebox
                    .bounds
                    .with_size(font.measure(text.trim_end(), params, false).into());
            }

            if let Some(line_bounds) = &mut line_bounds {
                *line_bounds += linebox.bounds;
            } else {
                line_bounds = Some(linebox.bounds);
            }

            box_count += 1;
        }

        let mut line_bounds = line_bounds.unwrap_or_default();

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
        if self.current_line_span.bullet && self.is_first_line && box_count > 0 {
            self.append_bullet(context, &self.current_line_span.clone());
        }

        box_count = 0;
        for linebox in self.boxes.get_mut(self.current_line..).unwrap() {
            // TODO: This attempts to keep text of multiple font sizes vertically
            // aligned correctly. It does not consider the baseline of the font,
            // which is information we don't have yet.
            let font_size_adjustment = self.max_font_size - linebox.bounds.height();

            if linebox.is_text_box() {
                linebox.bounds += Position::from((
                    left_adjustment + align_adjustment + (interim_adjustment * box_count),
                    font_size_adjustment,
                ));
            } else if linebox.is_bullet() {
                linebox.bounds += Position::from((Default::default(), font_size_adjustment));
            }

            box_count += 1;
        }

        if let Some((text, end, span)) = text {
            if box_count == 0 {
                self.append_text(&text[end..end], end, end, span);
            }
        }

        self.append_underlines();

        line_bounds +=
            Position::from((left_adjustment + align_adjustment, Twips::from_pixels(0.0)));
        line_bounds += Size::from((Twips::from_pixels(0.0), font_leading_adjustment));

        self.current_line = self.boxes.len();

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
    ///
    /// The `text`, `end`, and `span` parameters are for empty line insertion.
    /// `text` should be the text we are laying out, while `end` and `span` are
    /// the current positions into the text and format spans we are laying out.
    fn explicit_newline(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        text: &'a WStr,
        end: usize,
        span: &TextSpan,
    ) {
        self.fixup_line(context, false, true, Some((text, end, span)));

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
    ///
    /// The `text`, `end`, and `span` parameters are for empty line insertion.
    /// `text` should be the text we are laying out, while `end` and `span` are
    /// the current positions into the text and format spans we are laying out.
    fn newline(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        text: &'a WStr,
        end: usize,
        span: &TextSpan,
    ) {
        self.fixup_line(context, false, false, Some((text, end, span)));

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
        if self.is_start_of_line() {
            self.current_line_span = first_span.clone();
            self.max_font_size = Twips::from_pixels(first_span.size);
        } else {
            self.max_font_size = max(self.max_font_size, Twips::from_pixels(first_span.size));
        }
    }

    fn resolve_font(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        span: &TextSpan,
        is_device_font: bool,
    ) -> Option<Font<'gc>> {
        let library = context.library.library_for_movie_mut(self.movie.clone());

        // If this text field is set to use device fonts, fallback to using our embedded Noto Sans.
        // Note that the SWF can still contain a DefineFont tag with no glyphs/layout info in this case (see #451).
        // In an ideal world, device fonts would search for a matching font on the system and render it in some way.
        if let Some(font) = library
            .get_font_by_name(&span.font.to_utf8_lossy(), span.bold, span.italic)
            .filter(|f| !is_device_font && f.has_glyphs())
            .or_else(|| context.library.device_font())
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
    fn append_text(&mut self, text: &'a WStr, start: usize, end: usize, span: &TextSpan) {
        if self.effective_alignment() == swf::TextAlign::Justify {
            for word in text.split(b' ') {
                let word_start = word.offset_in(text).unwrap();
                let word_end = min(word_start + word.len() + 1, text.len());

                self.append_text_fragment(
                    &text[word_start..word_end],
                    start + word_start,
                    start + word_end,
                    span,
                );
            }
        } else {
            self.append_text_fragment(text, start, end, span);
        }
    }

    /// Append text fragments to the current line of the given layout context.
    ///
    /// This function bypasses the text fragmentation necessary for justify to
    /// work and it should only be called internally.
    fn append_text_fragment(&mut self, text: &'a WStr, start: usize, end: usize, span: &TextSpan) {
        let params = EvalParameters::from_span(span);
        let text_size = Size::from(self.font.unwrap().measure(text, params, false));
        let text_bounds = BoxBounds::from_position_and_size(self.cursor, text_size);
        let mut new_text = LayoutBox::from_text(start, end, self.font.unwrap(), span);

        new_text.bounds = text_bounds;

        self.cursor += Position::from((text_size.width(), Twips::default()));
        self.append_box(new_text);
    }

    /// Append a bullet to the start of the current line.
    ///
    /// The bullet will always be placed at the start of the current line. It
    /// should be appended after line fixup has completed, but before the text
    /// cursor is moved down.
    fn append_bullet(&mut self, context: &mut UpdateContext<'_, 'gc, '_>, span: &TextSpan) {
        let library = context.library.library_for_movie_mut(self.movie.clone());

        if let Some(bullet_font) = library
            .get_font_by_name(&span.font.to_utf8_lossy(), span.bold, span.italic)
            .filter(|f| f.has_glyphs())
            .or_else(|| context.library.device_font())
            .or(self.font)
        {
            let mut bullet_cursor = self.cursor;

            bullet_cursor.set_x(
                Twips::from_pixels(18.0)
                    + Self::left_alignment_offset_without_bullet(span, self.is_first_line),
            );

            let params = EvalParameters::from_span(span);
            let bullet = WStr::from_units(&[0x2022u16]);
            let text_size = Size::from(bullet_font.measure(bullet, params, false));
            let text_bounds = BoxBounds::from_position_and_size(bullet_cursor, text_size);
            let mut new_bullet = LayoutBox::from_bullet(bullet_font, span);

            new_bullet.bounds = text_bounds;

            self.append_box(new_bullet);
        }
    }

    /// Add a box to the current line of text.
    ///
    /// The box should have been positioned according to the current cursor
    /// position. It will be adjusted some time later to properly position it
    /// within the current layout box.
    fn append_box(&mut self, to_append: LayoutBox<'gc>) {
        self.boxes.push(to_append);
    }

    /// Calculate the left-align offset of a given line of text given the span
    /// active at the start of the line and if we're at the start of a
    /// paragraph.
    fn left_alignment_offset_without_bullet(span: &TextSpan, is_first_line: bool) -> Twips {
        if is_first_line {
            Twips::from_pixels(span.left_margin + span.block_indent + span.indent)
        } else {
            Twips::from_pixels(span.left_margin + span.block_indent)
        }
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
        } else {
            Self::left_alignment_offset_without_bullet(span, is_first_line)
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
        fs: &'a FormatSpans,
    ) -> (Vec<LayoutBox<'gc>>, BoxBounds<Twips>) {
        self.fixup_line(
            context,
            !self.has_line_break,
            true,
            fs.last_span()
                .map(|ls| (fs.displayed_text(), fs.displayed_text().len(), ls)),
        );

        (self.boxes, self.exterior_bounds.unwrap_or_default())
    }

    fn is_start_of_line(&self) -> bool {
        self.current_line >= self.boxes.len()
    }
}

/// A `LayoutBox` represents a single content box within a fully laid-out
/// `EditText`.
///
/// The content of each box is determined by `LayoutContent`.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LayoutBox<'gc> {
    /// The rectangle corresponding to the outer boundaries of the content box.
    bounds: BoxBounds<Twips>,

    /// What content is contained by the content box.
    content: LayoutContent<'gc>,
}

/// Represents different content modes of a given `LayoutBox`.
///
/// Currently, a `LayoutBox` can contain `Text`, `Bullet`s, or a `Drawing`.
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
        #[collect(require_static)]
        color: swf::Color,
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
        #[collect(require_static)]
        color: swf::Color,
    },

    /// A layout box containing a drawing.
    ///
    /// The drawing will be rendered with its origin at the position of the
    /// layout box's bounds. The size of those bounds do not affect the
    /// rendering of the drawing.
    Drawing(Drawing),
}

impl<'gc> LayoutBox<'gc> {
    /// Construct a text box for a text node.
    pub fn from_text(start: usize, end: usize, font: Font<'gc>, span: &TextSpan) -> Self {
        let params = EvalParameters::from_span(span);

        Self {
            bounds: Default::default(),
            content: LayoutContent::Text {
                start,
                end,
                text_format: span.get_text_format(),
                font,
                params,
                color: span.color.clone(),
            },
        }
    }

    /// Construct a bullet.
    pub fn from_bullet(font: Font<'gc>, span: &TextSpan) -> Self {
        let params = EvalParameters::from_span(span);

        Self {
            bounds: Default::default(),
            content: LayoutContent::Bullet {
                text_format: span.get_text_format(),
                font,
                params,
                color: span.color.clone(),
            },
        }
    }

    /// Construct a drawing.
    pub fn from_drawing(drawing: Drawing) -> Self {
        Self {
            bounds: Default::default(),
            content: LayoutContent::Drawing(drawing),
        }
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
        is_device_font: bool,
    ) -> (Vec<LayoutBox<'gc>>, BoxBounds<Twips>) {
        let mut layout_context = LayoutContext::new(movie, bounds, fs.displayed_text());

        for (span_start, _end, span_text, span) in fs.iter_spans() {
            if let Some(font) = layout_context.resolve_font(context, span, is_device_font) {
                layout_context.newspan(span);

                let params = EvalParameters::from_span(span);

                for text in span_text.split(&[b'\n', b'\r', b'\t'][..]) {
                    let slice_start = text.offset_in(span_text).unwrap();
                    let delimiter = if slice_start > 0 {
                        span_text
                            .get(slice_start - 1)
                            .and_then(|c| u8::try_from(c).ok())
                    } else {
                        None
                    };

                    match delimiter {
                        Some(b'\n' | b'\r') => {
                            layout_context.explicit_newline(context, text, 0, span)
                        }
                        Some(b'\t') => layout_context.tab(),
                        _ => {}
                    }

                    let start = span_start + slice_start;

                    let mut last_breakpoint = 0;

                    if is_word_wrap {
                        let (mut width, mut offset) = layout_context.wrap_dimensions(span);

                        while let Some(breakpoint) = font.wrap_line(
                            &text[last_breakpoint..],
                            params,
                            width,
                            offset,
                            layout_context.is_start_of_line(),
                        ) {
                            // This ensures that the space causing the line break
                            // is included in the line it broke.
                            let next_breakpoint = string_utils::next_char_boundary(
                                text,
                                last_breakpoint + breakpoint,
                            );

                            // If text doesn't fit at the start of a line, it
                            // won't fit on the next either, abort and put the
                            // whole text on the line (will be cut-off). This
                            // can happen for small text fields with single
                            // characters.
                            if breakpoint == 0 && layout_context.is_start_of_line() {
                                break;
                            } else if breakpoint == 0 {
                                layout_context.newline(context, text, next_breakpoint, span);

                                let next_dim = layout_context.wrap_dimensions(span);

                                width = next_dim.0;
                                offset = next_dim.1;

                                if last_breakpoint >= text.len() {
                                    break;
                                } else {
                                    continue;
                                }
                            }

                            layout_context.append_text(
                                &text[last_breakpoint..next_breakpoint],
                                start + last_breakpoint,
                                start + next_breakpoint,
                                span,
                            );

                            last_breakpoint = next_breakpoint;
                            if last_breakpoint >= text.len() {
                                break;
                            }

                            layout_context.newline(context, text, next_breakpoint, span);
                            let next_dim = layout_context.wrap_dimensions(span);

                            width = next_dim.0;
                            offset = next_dim.1;
                        }
                    }

                    let span_end = text.len();

                    if last_breakpoint < span_end {
                        layout_context.append_text(
                            &text[last_breakpoint..span_end],
                            start + last_breakpoint,
                            start + span_end,
                            span,
                        );
                    }
                }
            }
        }

        layout_context.end_layout(context, fs)
    }

    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    pub fn content(&self) -> &LayoutContent<'gc> {
        &self.content
    }

    /// Returns a reference to the text this box contains, as well as font
    /// rendering parameters, if the layout box has any.
    pub fn as_renderable_text<'a>(
        &self,
        text: &'a WStr,
    ) -> Option<(&'a WStr, &TextFormat, Font<'gc>, EvalParameters, swf::Color)> {
        match &self.content {
            LayoutContent::Text {
                start,
                end,
                text_format,
                font,
                params,
                color,
            } => Some((
                text.slice(*start..*end)?,
                text_format,
                *font,
                *params,
                swf::Color::from_rgb(color.to_rgb(), 0xFF),
            )),
            LayoutContent::Bullet {
                text_format,
                font,
                params,
                color,
            } => Some((
                WStr::from_units(&[0x2022u16]),
                text_format,
                *font,
                *params,
                swf::Color::from_rgb(color.to_rgb(), 0xFF),
            )),
            LayoutContent::Drawing(..) => None,
        }
    }

    /// Returns a reference to the drawing this box contains, if it has one.
    pub fn as_renderable_drawing(&self) -> Option<&Drawing> {
        match &self.content {
            LayoutContent::Text { .. } => None,
            LayoutContent::Bullet { .. } => None,
            LayoutContent::Drawing(drawing) => Some(drawing),
        }
    }

    pub fn is_text_box(&self) -> bool {
        matches!(&self.content, LayoutContent::Text { .. })
    }

    pub fn is_bullet(&self) -> bool {
        matches!(&self.content, LayoutContent::Bullet { .. })
    }
}

pub struct LayoutMetrics {
    pub ascent: Twips,
    pub descent: Twips,
    pub leading: Twips,

    pub width: Twips,
    pub height: Twips,

    pub x: Twips,
}
