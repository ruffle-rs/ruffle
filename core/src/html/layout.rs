//! Layout box structure

use crate::context::UpdateContext;
use crate::drawing::Drawing;
use crate::font::{EvalParameters, Font, FontType};
use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::string::{utils as string_utils, WStr};
use crate::tag_utils::SwfMovie;
use crate::DefaultFont;
use gc_arena::Collect;
use ruffle_render::shape_utils::DrawCommand;
use std::cmp::{max, min, Ordering};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::{Deref, Range};
use std::slice::Iter;
use std::sync::Arc;
use swf::{Point, Rectangle, Twips};

/// Draw an underline on a particular drawing.
///
/// This will not draw underlines shorter than a pixel in width.
fn draw_underline(
    drawing: &mut Drawing,
    starting_pos: Position<Twips>,
    width: Twips,
    color: swf::Color,
) {
    if width < Twips::ONE {
        return;
    }

    let ending_pos = starting_pos + Position::from((width, Twips::ZERO));

    drawing.set_line_style(Some(
        swf::LineStyle::new()
            .with_width(Twips::new(1))
            .with_color(color),
    ));
    drawing.draw_command(DrawCommand::MoveTo(Point::new(
        starting_pos.x(),
        starting_pos.y(),
    )));
    drawing.draw_command(DrawCommand::LineTo(Point::new(
        ending_pos.x(),
        ending_pos.y(),
    )));
}

/// Contains information relating to the current layout operation.
pub struct LayoutContext<'a, 'gc> {
    /// The movie this layout context is pulling fonts from.
    movie: Arc<SwfMovie>,

    /// Whether user input is allowed.
    is_input: bool,

    /// Whether word wrap is enabled.
    is_word_wrap: bool,

    /// Type of the font used by the text field.
    font_type: FontType,

    /// The position to put text into.
    ///
    /// We are laying out boxes so that the cursor is at their baseline.
    /// That way, they will be aligned properly when fixing up the line.
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

    /// The highest ascent observed within the current line.
    max_ascent: Twips,

    /// The highest descent observed within the current line.
    max_descent: Twips,

    /// The highest leading observed within the current line.
    max_leading: Twips,

    /// The growing list of layout lines to return when layout has finished.
    lines: Vec<LayoutLine<'gc>>,

    /// A counter used for indexing lines.
    /// Contains the index of the line currently being laid out.
    current_line_index: usize,

    /// A growing list of layout boxes that form the line currently being laid out.
    boxes: Vec<LayoutBox<'gc>>,

    /// The bounds of all laid-out text, excluding margins.
    ///
    /// None indicates that no bounds have yet to be calculated. If the layout
    /// ends without a line having been generated, the default bounding box
    /// should be used.
    bounds: Option<BoxBounds<Twips>>,

    /// Bounds used to calculate text metrics, i.e. its width and height.
    ///
    /// None indicates that no bounds have yet to be calculated. If the layout
    /// ends without a line having been generated, the default bounding box
    /// should be used.
    text_size_bounds: Option<BoxBounds<Twips>>,

    /// Whether we are laying out the first line of a paragraph.
    is_first_line: bool,

    /// Whether we encountered a line break of any kind during layout.
    ///
    /// Flash always applies at least one count of the line leading to the
    /// total text bounds of the laid-out text, even if there are no line
    /// breaks to host that leading. This flags that we need to add leading to
    /// the singular line if we have yet to process a newline.
    has_line_break: bool,

    /// The right margin of the first span in the current line.
    current_line_span: TextSpan,

    /// The total width of the text field being laid out.
    max_bounds: Twips,
}

impl<'a, 'gc> LayoutContext<'a, 'gc> {
    fn new(
        movie: Arc<SwfMovie>,
        max_bounds: Twips,
        text: &'a WStr,
        is_input: bool,
        is_word_wrap: bool,
        font_type: FontType,
    ) -> Self {
        Self {
            movie,
            cursor: Default::default(),
            font: None,
            text,
            max_font_size: Default::default(),
            max_ascent: Default::default(),
            max_descent: Default::default(),
            max_leading: Default::default(),
            lines: Vec::new(),
            current_line_index: 0,
            boxes: Vec::new(),
            bounds: None,
            text_size_bounds: None,
            is_first_line: true,
            has_line_break: false,
            current_line_span: Default::default(),
            max_bounds,
            is_input,
            is_word_wrap,
            font_type,
        }
    }

    fn lay_out_spans(&mut self, context: &mut UpdateContext<'gc>, fs: &'a FormatSpans) {
        for (span_start, _end, span_text, span) in fs.iter_spans() {
            self.lay_out_span(context, span_start, span_text, span);
        }
    }

    fn lay_out_span(
        &mut self,
        context: &mut UpdateContext<'gc>,
        span_start: usize,
        span_text: &'a WStr,
        span: &TextSpan,
    ) {
        let font = self.resolve_font(context, span);
        self.font = Some(font);
        self.newspan(span);

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
                    self.newline(context, span_start + slice_start - 1, span, true)
                }
                Some(b'\t') => self.tab(),
                _ => {}
            }

            let start = span_start + slice_start;

            let mut last_breakpoint = 0;

            if self.is_word_wrap {
                let (mut width, mut offset) = self.wrap_dimensions(span);

                while let Some(breakpoint) = font.wrap_line(
                    &text[last_breakpoint..],
                    params,
                    width,
                    offset,
                    self.is_start_of_line(),
                ) {
                    // This ensures that the space causing the line break
                    // is included in the line it broke.
                    let next_breakpoint =
                        string_utils::next_char_boundary(text, last_breakpoint + breakpoint);

                    // If text doesn't fit at the start of a line, it
                    // won't fit on the next either, abort and put the
                    // whole text on the line (will be cut-off). This
                    // can happen for small text fields with single
                    // characters.
                    if breakpoint == 0 && self.is_start_of_line() {
                        break;
                    } else if breakpoint == 0 {
                        self.newline(context, start + next_breakpoint, span, false);

                        let next_dim = self.wrap_dimensions(span);

                        width = next_dim.0;
                        offset = next_dim.1;

                        if last_breakpoint >= text.len() {
                            break;
                        } else {
                            continue;
                        }
                    }

                    self.append_text(
                        &text[last_breakpoint..next_breakpoint],
                        start + last_breakpoint,
                        start + next_breakpoint,
                        span,
                    );

                    last_breakpoint = next_breakpoint;
                    if last_breakpoint >= text.len() {
                        break;
                    }

                    self.newline(context, start + next_breakpoint, span, false);
                    let next_dim = self.wrap_dimensions(span);

                    width = next_dim.0;
                    offset = next_dim.1;
                }
            }

            let span_end = text.len();

            if last_breakpoint < span_end {
                self.append_text(
                    &text[last_breakpoint..span_end],
                    start + last_breakpoint,
                    start + span_end,
                    span,
                );
            }
        }
    }

    /// Calculate the line-to-line leading present on this line.
    fn line_leading_adjustment(&self) -> Twips {
        // Flash Player ignores font-provided leading.
        Twips::from_pixels(self.current_line_span.leading)
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
        let mut underline_color: Option<swf::Color> = None;
        let mut line_drawing = Drawing::new();
        let mut has_underline: bool = false;

        for linebox in self.boxes.iter() {
            if linebox.is_text_box() {
                if let Some((_t, tf, font, params, color)) = linebox.as_renderable_text(self.text) {
                    let underline_baseline =
                        font.get_baseline_for_height(params.height()) + Twips::from_pixels(2.0);
                    let mut line_extended = false;

                    if let (Some(starting_pos), Some(underline_color)) =
                        (starting_pos, underline_color)
                    {
                        if tf.underline.unwrap_or(false)
                            && underline_baseline + linebox.bounds().origin().y()
                                == starting_pos.y()
                            && underline_color == color
                        {
                            //Underline is at the same baseline, extend it
                            current_width = Some(linebox.bounds().extent_x() - starting_pos.x());

                            line_extended = true;
                        }
                    }

                    if !line_extended {
                        //For whatever reason, we cannot extend the current underline.
                        //This can happen if we don't have an underline to extend, the
                        //underlines don't match, or this span doesn't call for one.
                        if let (Some(pos), Some(width), Some(color)) =
                            (starting_pos, current_width, underline_color)
                        {
                            draw_underline(&mut line_drawing, pos, width, color);
                            has_underline = true;
                            starting_pos = None;
                            current_width = None;
                            underline_color = None;
                        }

                        if tf.underline.unwrap_or(false) {
                            starting_pos = Some(
                                linebox.bounds().origin()
                                    + Position::from((Twips::ZERO, underline_baseline)),
                            );
                            current_width = Some(linebox.bounds().width());
                            underline_color = Some(color);
                        }
                    }
                }
            }
        }

        if let (Some(starting_pos), Some(current_width), Some(underline_color)) =
            (starting_pos, current_width, underline_color)
        {
            draw_underline(
                &mut line_drawing,
                starting_pos,
                current_width,
                underline_color,
            );
            has_underline = true;
        }

        if has_underline {
            let pos = self.last_box_end_position();
            self.append_box(LayoutBox::from_drawing(pos, line_drawing));
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
        context: &mut UpdateContext<'gc>,
        last_line: bool,
        final_line_of_para: bool,
        end: usize,
        span: &TextSpan,
    ) {
        if self.boxes.is_empty() {
            self.append_text(WStr::empty(), end, end, span);
        }
        let first_box = self
            .boxes
            .first()
            .expect("each line must have at least one box");
        let is_line_empty = first_box.start() == end;

        let mut line_size_bounds = None;
        let mut box_count: i32 = 0;
        for linebox in self.boxes.iter_mut() {
            let (text, _tf, font, params, _color) =
                linebox.as_renderable_text(self.text).expect("text");

            // Flash ignores trailing spaces when aligning lines, so should we
            // TODO This behavior is dependent on SWF version
            if self.current_line_span.align != swf::TextAlign::Left {
                linebox.bounds = linebox
                    .bounds
                    .with_width(font.measure(text.trim_end(), params));
            }

            Self::extend_bounds(&mut line_size_bounds, linebox.bounds);

            box_count += 1;
        }

        let mut line_size_bounds = line_size_bounds.unwrap_or_default();

        let left_adjustment =
            Self::left_alignment_offset(&self.current_line_span, self.is_first_line);
        let right_adjustment = Twips::from_pixels(self.current_line_span.right_margin);

        let misalignment =
            self.max_bounds - left_adjustment - right_adjustment - line_size_bounds.width();
        let align_adjustment = max(
            match self.effective_alignment() {
                swf::TextAlign::Left | swf::TextAlign::Justify => Default::default(),
                swf::TextAlign::Center => (misalignment) / 2,
                swf::TextAlign::Right => misalignment,
            },
            Twips::ZERO,
        );
        let interim_adjustment = max(
            if !final_line_of_para && self.effective_alignment() == swf::TextAlign::Justify {
                misalignment / max(box_count.saturating_sub(1), 1)
            } else {
                Twips::ZERO
            },
            Twips::ZERO,
        );

        if self.current_line_span.bullet && self.is_first_line && box_count > 0 {
            self.append_bullet(context, &self.current_line_span.clone());
        }

        let baseline_adjustment = self.max_ascent;

        box_count = 0;
        for layout_box in self.boxes.iter_mut() {
            if layout_box.is_text_box() {
                let position = Position::from((
                    left_adjustment + align_adjustment + (interim_adjustment * box_count),
                    baseline_adjustment,
                ));
                layout_box.bounds += position;
            } else if layout_box.is_bullet() {
                let position = Position::from((Twips::ZERO, baseline_adjustment));
                layout_box.bounds += position;
            }

            box_count += 1;
        }

        self.append_underlines();

        line_size_bounds +=
            Position::from((left_adjustment + align_adjustment, baseline_adjustment));

        if self.current_line_index == 0 {
            // The very first line always gets the leading.
            line_size_bounds += Size::from((Twips::ZERO, self.max_leading));
        }

        if !self.is_input && is_line_empty && last_line {
            // For non-input fields, skip the last line if it's empty.
            // For input fields, we have to take the empty line into account,
            // otherwise it wouldn't be possible to click there to input text.
        } else {
            Self::extend_bounds(&mut self.text_size_bounds, line_size_bounds);
        }

        self.flush_line(end);
    }

    fn flush_line(&mut self, end: usize) {
        if self.boxes.is_empty() {
            return;
        }

        let boxes = mem::take(&mut self.boxes);
        let first_box = boxes.first().unwrap();
        let start = first_box.start();
        let bounds = boxes
            .iter()
            .filter(|b| b.is_text_box())
            .fold(first_box.bounds, |bounds, b| bounds + b.bounds);

        // Update last line's end position to take into account the delimiter.
        // It's easier to do it here, but maybe after some refactors this update
        // will not be needed, and the end position will be calculated correctly.
        if let Some(last_line) = self.lines.last_mut() {
            last_line.end = start;
        }

        self.lines.push(LayoutLine {
            index: self.current_line_index,
            bounds,
            start,
            end,
            boxes,
            ascent: self.max_ascent,
            descent: self.max_descent,
            leading: self.max_leading,
        });
        self.current_line_index += 1;

        // Update layout bounds
        Self::extend_bounds(&mut self.bounds, bounds);
    }

    fn extend_bounds(bounds: &mut Option<BoxBounds<Twips>>, to_extend: BoxBounds<Twips>) {
        if let Some(b) = bounds {
            *b += to_extend;
        } else {
            *bounds = Some(to_extend);
        }
    }

    /// Adjust the text layout cursor down to the next line.
    ///
    /// This function will also adjust any layout boxes on the current line to
    /// their correct alignment and indentation.
    ///
    /// The `text`, `end`, and `span` parameters are for empty line insertion.
    /// `text` should be the text we are laying out, while `end` and `span` are
    /// the current positions into the text and format spans we are laying out.
    ///
    /// The parameter `end_of_para` specifies whether the line was the last line
    /// of the current paragraph (i.e. it contained an explicit newline).
    fn newline(
        &mut self,
        context: &mut UpdateContext<'gc>,
        end: usize,
        span: &TextSpan,
        end_of_para: bool,
    ) {
        self.fixup_line(context, false, end_of_para, end, span);

        self.cursor.set_x(Twips::ZERO);
        self.cursor += (
            Twips::ZERO,
            self.max_ascent + self.max_descent + self.line_leading_adjustment(),
        )
            .into();

        self.is_first_line = end_of_para;
        self.has_line_break = true;

        let font_size = Twips::from_pixels(self.current_line_span.font.size);
        let font = self.font.unwrap();
        self.max_font_size = font_size;
        self.max_ascent = font.get_baseline_for_height(font_size);
        self.max_descent = font.get_descent_for_height(font_size);
        self.max_leading = Twips::from_pixels(span.leading);
    }

    /// Adjust the text layout cursor in response to a tab.
    ///
    /// Tabs can do two separate things in Flash, depending on whether
    /// tab stops have been manually determined. If they have been, then the
    /// text cursor goes to the next closest tab stop that has not yet been
    /// passed, or if no such stop exists, tabs do nothing. If no tab stops
    /// exist, then the cursor is advanced to some position modulo the natural
    /// tab index.
    fn tab(&mut self) {
        if self.current_line_span.tab_stops.is_empty() {
            let modulo_factor = Twips::from_pixels(self.current_line_span.font.size * 2.7);
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
        let font_size = Twips::from_pixels(first_span.font.size);
        let font = self.font.unwrap();
        let ascent = font.get_baseline_for_height(font_size);
        let descent = font.get_descent_for_height(font_size);
        let leading = Twips::from_pixels(first_span.leading);
        if self.is_start_of_line() {
            self.current_line_span = first_span.clone();
            self.max_font_size = font_size;
            self.max_ascent = ascent;
            self.max_descent = descent;
            self.max_leading = leading;
        } else {
            self.max_font_size = self.max_font_size.max(font_size);
            self.max_ascent = self.max_ascent.max(ascent);
            self.max_descent = self.max_descent.max(descent);
            self.max_leading = self.max_leading.max(leading);
        }
    }

    fn resolve_font(&mut self, context: &mut UpdateContext<'gc>, span: &TextSpan) -> Font<'gc> {
        fn new_empty_font<'gc>(
            context: &mut UpdateContext<'gc>,
            span: &TextSpan,
            font_type: FontType,
        ) -> Font<'gc> {
            Font::empty_font(
                context.gc(),
                &span.font.face.to_utf8_lossy(),
                span.style.bold,
                span.style.italic,
                font_type,
            )
        }

        fn describe_font(span: &TextSpan) -> String {
            let bold_suffix = if span.style.bold { ", bold" } else { "" };
            let italic_suffix = if span.style.italic { ", italic" } else { "" };
            format!(
                "{}{}{}",
                span.font.face.to_utf8_lossy(),
                bold_suffix,
                italic_suffix
            )
        }

        let font_name = span.font.face.to_utf8_lossy();

        // Note that the SWF can still contain a DefineFont tag with no glyphs/layout info in this case (see #451).
        // In an ideal world, device fonts would search for a matching font on the system and render it in some way.
        if self.font_type != FontType::Device {
            if let Some(font) = context
                .library
                .get_embedded_font_by_name(
                    &font_name,
                    self.font_type,
                    span.style.bold,
                    span.style.italic,
                    Some(self.movie.clone()),
                )
                .filter(|f| f.has_glyphs())
            {
                return font;
            }
            // TODO: If set to use embedded fonts and we couldn't find any matching font, show nothing
            // However - at time of writing, we don't support DefineFont4. If we matched this behaviour,
            // then a bunch of SWFs would just show no text suddenly.
            // return new_empty_font(context, span, self.font_type);
        }

        // Check if the font name is one of the known default fonts.
        if let Some(default_font) = match font_name.deref() {
            "_serif" => Some(DefaultFont::Serif),
            "_sans" => Some(DefaultFont::Sans),
            "_typewriter" => Some(DefaultFont::Typewriter),
            "_ゴシック" => Some(DefaultFont::JapaneseGothic),
            "_等幅" => Some(DefaultFont::JapaneseGothicMono),
            "_明朝" => Some(DefaultFont::JapaneseMincho),
            _ => None,
        } {
            if let Some(&font) = context
                .library
                .default_font(
                    default_font,
                    span.style.bold,
                    span.style.italic,
                    context.ui,
                    context.renderer,
                    context.gc_context,
                )
                .first()
            {
                return font;
            } else {
                let font_desc = describe_font(span);
                tracing::error!(
                    "Known default device font not found: {font_desc}, text will be missing"
                );
                return new_empty_font(context, span, self.font_type);
            }
        }

        if let Some(font) = context.library.get_or_load_device_font(
            &font_name,
            span.style.bold,
            span.style.italic,
            context.ui,
            context.renderer,
            context.gc_context,
        ) {
            return font;
        }

        // TODO: handle multiple fonts for a definition, each covering different sets of glyphs

        // At this point, the font name was neither one of the default
        // fonts nor matched any device font. We explicitly handle some of the
        // well-known aliases for the default fonts for better compatibility
        // with devices that don't have those fonts installed. As a last resort
        // we fall back to using sans (like Flash).
        let default_font = match font_name.deref() {
            "Times New Roman" => DefaultFont::Serif,
            "Arial" => DefaultFont::Sans,
            "Courier New" => DefaultFont::Typewriter,
            _ => {
                if font_name.contains("Ming") || font_name.contains('明') {
                    DefaultFont::JapaneseMincho
                } else {
                    DefaultFont::Sans
                }
            }
        };

        if let Some(&font) = context
            .library
            .default_font(
                default_font,
                span.style.bold,
                span.style.italic,
                context.ui,
                context.renderer,
                context.gc_context,
            )
            .first()
        {
            font
        } else {
            let font_desc = describe_font(span);
            tracing::error!(
                "Fallback font not found ({default_font:?}) for: {font_desc}, text will be missing"
            );
            new_empty_font(context, span, self.font_type)
        }
    }

    /// Append text to the current line of the ongoing layout operation.
    ///
    /// The text given may or may not be separated into fragments, depending on
    /// what the layout calls for.
    fn append_text(&mut self, text: &'a WStr, start: usize, end: usize, span: &TextSpan) {
        let empty = start == end;
        if !empty && self.effective_alignment() == swf::TextAlign::Justify {
            for word in text.split(b' ') {
                let word_start = word.offset_in(text).unwrap();
                let word_end = min(word_start + word.len() + 1, text.len());

                if word_start == word_end {
                    // Do not append empty boxes
                    continue;
                }

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
    /// work, and it should only be called internally.
    fn append_text_fragment(&mut self, text: &'a WStr, start: usize, end: usize, span: &TextSpan) {
        let font = self.font.expect("text fragment requires a font");
        let params = EvalParameters::from_span(span);
        let ascent = font.get_baseline_for_height(params.height());
        let descent = font.get_descent_for_height(params.height());
        let text_width = font.measure(text, params);
        let box_origin = self.cursor - (Twips::ZERO, ascent).into();

        let mut new_box = LayoutBox::from_text(text, start, end, font, span);
        new_box.bounds = BoxBounds::from_position_and_size(
            box_origin,
            Size::from((text_width, ascent + descent)),
        );

        self.cursor += (text_width, Twips::ZERO).into();
        self.append_box(new_box);
    }

    /// Append a bullet to the start of the current line.
    ///
    /// The bullet will always be placed at the start of the current line. It
    /// should be appended after line fixup has completed, but before the text
    /// cursor is moved down.
    fn append_bullet(&mut self, context: &mut UpdateContext<'gc>, span: &TextSpan) {
        let bullet_font = self.resolve_font(context, span);
        let mut bullet_cursor = self.cursor;

        bullet_cursor.set_x(
            Twips::from_pixels(18.0)
                + Self::left_alignment_offset_without_bullet(span, self.is_first_line),
        );

        let params = EvalParameters::from_span(span);
        let ascent = bullet_font.get_baseline_for_height(params.height());
        let descent = bullet_font.get_descent_for_height(params.height());
        let bullet = WStr::from_units(&[0x2022u16]);
        let text_width = bullet_font.measure(bullet, params);
        let box_origin = bullet_cursor - (Twips::ZERO, ascent).into();

        let pos = self.last_box_end_position();
        let mut new_bullet = LayoutBox::from_bullet(pos, bullet_font, span);
        new_bullet.bounds = BoxBounds::from_position_and_size(
            box_origin,
            Size::from((text_width, ascent + descent)),
        );

        self.append_box(new_bullet);
    }

    fn last_box_end_position(&self) -> usize {
        self.boxes.last().map(|b| b.end()).unwrap_or(0)
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
    fn end_layout(mut self, context: &mut UpdateContext<'gc>, fs: &'a FormatSpans) -> Layout<'gc> {
        let last_span = fs.last_span().expect("At least one span should be present");
        self.fixup_line(context, true, true, fs.displayed_text().len(), last_span);

        let text_size = self.text_size_bounds.unwrap_or_default();
        Layout {
            bounds: self.bounds.unwrap_or_default(),
            text_size: Size::from((text_size.width(), text_size.height())),
            lines: self.lines,
        }
    }

    fn is_start_of_line(&self) -> bool {
        self.boxes.is_empty()
    }
}

/// Construct a new layout from text spans.
pub fn lower_from_text_spans<'gc>(
    fs: &FormatSpans,
    context: &mut UpdateContext<'gc>,
    movie: Arc<SwfMovie>,
    requested_width: Option<Twips>,
    is_input: bool,
    is_word_wrap: bool,
    font_type: FontType,
) -> Layout<'gc> {
    let requested_width = requested_width.unwrap_or_else(|| {
        // When we don't know the width of the text field, we have to lay out
        // text two times: the first time to calculate the proper width, and
        // the second time to lay out text knowing the proper width.
        let layout = lower_from_text_spans_known_width(
            fs,
            context,
            movie.clone(),
            Twips::ZERO,
            is_input,
            false,
            font_type,
        );
        let max_width = layout
            .lines()
            .iter()
            .map(|line| line.bounds().width())
            .max();
        max_width.unwrap_or_default()
    });
    lower_from_text_spans_known_width(
        fs,
        context,
        movie,
        requested_width,
        is_input,
        is_word_wrap,
        font_type,
    )
}

fn lower_from_text_spans_known_width<'gc>(
    fs: &FormatSpans,
    context: &mut UpdateContext<'gc>,
    movie: Arc<SwfMovie>,
    bounds: Twips,
    is_input: bool,
    is_word_wrap: bool,
    font_type: FontType,
) -> Layout<'gc> {
    let mut layout_context = LayoutContext::new(
        movie,
        bounds,
        fs.displayed_text(),
        is_input,
        is_word_wrap,
        font_type,
    );

    layout_context.lay_out_spans(context, fs);

    layout_context.end_layout(context, fs)
}

/// A `Layout` represents a fully laid-out text field.
/// It consists of [`LayoutLine`]s.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Layout<'gc> {
    #[collect(require_static)]
    bounds: BoxBounds<Twips>,

    #[collect(require_static)]
    text_size: Size<Twips>,

    lines: Vec<LayoutLine<'gc>>,
}

impl<'gc> Layout<'gc> {
    /// Bounds of this layout, i.e. a union of bounds of all layout boxes.
    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    /// Text size of this layout.
    pub fn text_size(&self) -> Size<Twips> {
        self.text_size
    }

    pub fn lines(&self) -> &Vec<LayoutLine<'gc>> {
        &self.lines
    }

    pub fn boxes_iter(&self) -> LayoutBoxIter<'_, 'gc> {
        LayoutBoxIter {
            lines_iter: self.lines.iter(),
            boxes_iter: None,
        }
    }

    pub fn find_line_index_by_position(&self, position: usize) -> Option<usize> {
        let result = self.lines.binary_search_by(|probe| {
            if probe.end <= position {
                Ordering::Less
            } else if position < probe.start {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        result.ok()
    }

    /// Returns the index of the line which is
    /// positioned at the given y coordinate.
    ///
    /// If the coordinate is at line leading, the line above is returned.
    /// If the coordinate is above all lines, [`Result::Err`] is returned
    /// with the index of the first line.
    /// If the coordinate is below all lines, [`Result::Err`] is returned
    /// with the index of the last line.
    pub fn find_line_index_by_y(&self, y: Twips) -> Result<usize, usize> {
        let lines_len = self.lines.len();
        if y < Twips::ZERO || lines_len == 0 {
            return Err(0);
        }

        let result = self.lines.binary_search_by(|probe| {
            let bounds = probe.bounds();
            if bounds.extent_y() + probe.leading() <= y {
                Ordering::Less
            } else if y < bounds.offset_y() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        let line = result.unwrap_or_else(|i| i);
        let max_line = lines_len.saturating_sub(1);
        if line <= max_line {
            Ok(line)
        } else {
            Err(max_line)
        }
    }

    /// Returns char bounds of the given char relative to this layout.
    pub fn char_bounds(&self, position: usize) -> Option<Rectangle<Twips>> {
        let line_index = self.find_line_index_by_position(position)?;
        let line = self.lines.get(line_index)?;
        line.char_bounds(position)
    }
}

/// A `LayoutLine` represents a single line of text.
/// It consists of [`LayoutBox`]es.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LayoutLine<'gc> {
    /// Zero-based index of the line in the text.
    #[collect(require_static)]
    index: usize,

    /// Line bounds.
    ///
    /// They represent the area where this line is drawn.
    /// Their height will be equal to the largest character
    /// height (ascent + descent) on this line, leading is not included.
    #[collect(require_static)]
    bounds: BoxBounds<Twips>,

    /// The start position of the line (inclusive).
    start: usize,

    /// The end position of the line (exclusive).
    /// This position includes the line delimiter.
    end: usize,

    /// The highest ascent observed within this line.
    #[collect(require_static)]
    ascent: Twips,

    /// The highest descent observed within this line.
    #[collect(require_static)]
    descent: Twips,

    /// The highest leading observed within this line.
    #[collect(require_static)]
    leading: Twips,

    /// Layout boxes contained within this line.
    boxes: Vec<LayoutBox<'gc>>,
}

impl<'gc> LayoutLine<'gc> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn ascent(&self) -> Twips {
        self.ascent
    }

    pub fn descent(&self) -> Twips {
        self.descent
    }

    pub fn leading(&self) -> Twips {
        self.leading
    }

    pub fn len(&self) -> usize {
        self.end() - self.start()
    }

    pub fn text_range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn offset_y(&self) -> Twips {
        self.bounds().offset_y()
    }

    pub fn extent_y(&self) -> Twips {
        self.bounds().extent_y()
    }

    pub fn boxes_iter(&self) -> Iter<'_, LayoutBox<'gc>> {
        self.boxes.iter()
    }

    pub fn find_box_index_by_position(&self, position: usize) -> Option<usize> {
        let result = self.boxes.binary_search_by(|probe| {
            if probe.end() <= position {
                Ordering::Less
            } else if position < probe.start() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        result.ok()
    }

    /// Returns x-axis char bounds of the given char relative to the whole layout.
    pub fn char_x_bounds(&self, position: usize) -> Option<(Twips, Twips)> {
        let box_index = self.find_box_index_by_position(position)?;
        let layout_box = self.boxes.get(box_index)?;
        let (start, mut end) = layout_box.char_x_bounds(position)?;

        // If we're getting bounds of a space in justified text,
        // we have to take into account that the character is stretched,
        // and its bounds will end where the next character starts.
        // If it's not a space or the text is not justified, it won't change the value.
        // TODO [KJ] We need to test this behavior with letter spacing or kerning enabled.
        if layout_box.end() == position + 1 {
            if let Some(next_box) = self.boxes.get(box_index + 1) {
                if let Some(next_start) = next_box.char_x_bounds(position + 1).map(|(s, _)| s) {
                    end = next_start;
                }
            }
        }

        Some((start, end))
    }

    /// Returns char bounds of the given char relative to the whole layout.
    pub fn char_bounds(&self, position: usize) -> Option<Rectangle<Twips>> {
        let (x_min, x_max) = self.char_x_bounds(position)?;
        let line_bounds = self.bounds();
        Some(Rectangle {
            x_min,
            x_max,
            y_min: line_bounds.offset_y(),
            y_max: line_bounds.extent_y(),
        })
    }
}

/// A `LayoutBox` represents a single content box within a layout.
///
/// The content of each box is determined by [`LayoutContent`].
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LayoutBox<'gc> {
    /// Outer bounds of this layout box.
    ///
    /// The width of those bounds is equal to the width of the glyphs inside,
    /// whereas the height is equal to the font height (ascent + descent).
    ///
    /// TODO Currently, only text boxes have meaningful bounds.
    #[collect(require_static)]
    bounds: BoxBounds<Twips>,

    /// What content is contained by the content box.
    content: LayoutContent<'gc>,
}

/// Represents different content modes of a given `LayoutBox`.
///
/// Currently, a `LayoutBox` can contain `Text`, `Bullet`s, or a `Drawing`.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum LayoutContent<'gc> {
    /// A layout box containing some part of a text span.
    ///
    /// The text is assumed to be pulled from the same `FormatSpans` that
    /// generated this layout box.
    Text {
        /// The start position of the text to render (inclusive).
        start: usize,

        /// The end position of the text to render (exclusive).
        end: usize,

        /// The formatting options for the text box.
        #[collect(require_static)]
        text_format: TextFormat,

        /// The font that was resolved at the time of layout for this text.
        font: Font<'gc>,

        /// All parameters used to evaluate the font at the time of layout for
        /// this text.
        #[collect(require_static)]
        params: EvalParameters,

        /// The color to render the font with.
        #[collect(require_static)]
        color: swf::Color,

        /// List of end positions (relative to this box) for each character.
        ///
        /// By having this here, we do not have to reevaluate the font
        /// each time we want to get the position of a character,
        /// and we can use this data along with layout box bounds to
        /// calculate character bounds.
        ///
        /// For instance, for the text "hello", this field may contain:
        ///
        /// ```text
        /// [100, 200, 250, 300, 400]
        /// ```
        #[collect(require_static)]
        char_end_pos: Vec<Twips>,
    },

    /// A layout box containing a bullet.
    ///
    /// This is almost identical to `Text`, but the text contents are assumed
    /// to be U+2022.
    Bullet {
        /// The position of the bullet.
        position: usize,

        /// The formatting options for the text box.
        #[collect(require_static)]
        text_format: TextFormat,

        /// The font that was resolved at the time of layout for this text.
        font: Font<'gc>,

        /// All parameters used to evaluate the font at the time of layout for
        /// this text.
        #[collect(require_static)]
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
    Drawing {
        /// The position of the drawing in text.
        position: usize,

        #[collect(require_static)]
        drawing: Drawing,
    },
}

impl Debug for LayoutContent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutContent::Text { start, end, .. } => f
                .debug_struct("Text")
                .field("start", start)
                .field("end", end)
                .finish(),
            LayoutContent::Bullet { position, .. } => f
                .debug_struct("Bullet")
                .field("position", position)
                .finish(),
            LayoutContent::Drawing { position, .. } => f
                .debug_struct("Drawing")
                .field("position", position)
                .finish(),
        }
    }
}

impl<'gc> LayoutBox<'gc> {
    /// Construct a text box for a text node.
    pub fn from_text(
        text: &WStr,
        start: usize,
        end: usize,
        font: Font<'gc>,
        span: &TextSpan,
    ) -> Self {
        let params = EvalParameters::from_span(span);
        let mut char_end_pos = Vec::with_capacity(end - start);

        font.evaluate(text, Default::default(), params, |_, _, _, advance, x| {
            char_end_pos.push(x + advance);
        });

        Self {
            bounds: Default::default(),
            content: LayoutContent::Text {
                start,
                end,
                text_format: span.get_text_format(),
                font,
                params,
                color: span.font.color,
                char_end_pos,
            },
        }
    }

    /// Construct a bullet.
    pub fn from_bullet(position: usize, font: Font<'gc>, span: &TextSpan) -> Self {
        let params = EvalParameters::from_span(span);

        Self {
            bounds: Default::default(),
            content: LayoutContent::Bullet {
                position,
                text_format: span.get_text_format(),
                font,
                params,
                color: span.font.color,
            },
        }
    }

    /// Construct a drawing.
    pub fn from_drawing(position: usize, drawing: Drawing) -> Self {
        Self {
            bounds: Default::default(),
            content: LayoutContent::Drawing { position, drawing },
        }
    }

    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    pub fn content(&self) -> &LayoutContent<'gc> {
        &self.content
    }

    pub fn is_link(&self) -> bool {
        match &self.content {
            LayoutContent::Text {
                text_format: TextFormat { url: Some(url), .. },
                ..
            } => url.len() > 0,
            LayoutContent::Bullet {
                text_format: TextFormat { url: Some(url), .. },
                ..
            } => url.len() > 0,
            _ => false,
        }
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
                ..
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
                ..
            } => Some((
                WStr::from_units(&[0x2022u16]),
                text_format,
                *font,
                *params,
                swf::Color::from_rgb(color.to_rgb(), 0xFF),
            )),
            LayoutContent::Drawing { .. } => None,
        }
    }

    /// Returns a reference to the drawing this box contains, if it has one.
    pub fn as_renderable_drawing(&self) -> Option<&Drawing> {
        match &self.content {
            LayoutContent::Text { .. } => None,
            LayoutContent::Bullet { .. } => None,
            LayoutContent::Drawing { drawing, .. } => Some(drawing),
        }
    }

    pub fn is_text_box(&self) -> bool {
        matches!(&self.content, LayoutContent::Text { .. })
    }

    pub fn is_bullet(&self) -> bool {
        matches!(&self.content, LayoutContent::Bullet { .. })
    }

    pub fn start(&self) -> usize {
        match &self.content {
            LayoutContent::Text { start, .. } => *start,
            LayoutContent::Bullet { position, .. } => *position,
            LayoutContent::Drawing { position, .. } => *position,
        }
    }

    pub fn end(&self) -> usize {
        match &self.content {
            LayoutContent::Text { end, .. } => *end,
            LayoutContent::Bullet { position, .. } => *position,
            LayoutContent::Drawing { position, .. } => *position,
        }
    }

    /// Return x-axis char bounds of the given char relative to the whole layout.
    pub fn char_x_bounds(&self, position: usize) -> Option<(Twips, Twips)> {
        let relative_position = position.checked_sub(self.start())?;

        let LayoutContent::Text { char_end_pos, .. } = &self.content else {
            return None;
        };

        let origin_x = self.bounds().origin().x();

        Some(if relative_position == 0 {
            (origin_x, origin_x + *char_end_pos.get(0)?)
        } else {
            (
                origin_x + *char_end_pos.get(relative_position - 1)?,
                origin_x + *char_end_pos.get(relative_position)?,
            )
        })
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

pub struct LayoutBoxIter<'layout, 'gc> {
    lines_iter: Iter<'layout, LayoutLine<'gc>>,
    boxes_iter: Option<Iter<'layout, LayoutBox<'gc>>>,
}

impl<'layout, 'gc> Iterator for LayoutBoxIter<'layout, 'gc> {
    type Item = &'layout LayoutBox<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(boxes_iter) = self.boxes_iter.as_mut() {
                if let Some(next) = boxes_iter.next() {
                    return Some(next);
                }
            }

            if let Some(next_line) = self.lines_iter.next() {
                self.boxes_iter = Some(next_line.boxes_iter());
            } else {
                self.boxes_iter = None;
                // No more lines
                return None;
            }
        }
    }
}
