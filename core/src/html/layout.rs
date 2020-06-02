//! Layout box structure

use crate::context::UpdateContext;
use crate::font::{round_down_to_pixel, Font};
use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cmp::{max, min};
use std::sync::Arc;
use swf::Twips;

/// Contains information relating to the current layout operation.
pub struct LayoutContext<'a, 'gc> {
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

    /// All layout boxes in the current line being laid out.
    current_line: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The right margin of the first span in the current line.
    current_line_span: TextSpan,

    /// The total width of the text field being laid out.
    max_bounds: Twips,
}

impl<'a, 'gc> LayoutContext<'a, 'gc> {
    fn new(max_bounds: Twips, text: &'a str) -> Self {
        Self {
            cursor: Default::default(),
            font: None,
            text,
            max_font_size: Default::default(),
            first_box: None,
            last_box: None,
            exterior_bounds: None,
            is_first_line: true,
            current_line: None,
            current_line_span: Default::default(),
            max_bounds,
        }
    }

    fn cursor(&self) -> &Position<Twips> {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut Position<Twips> {
        &mut self.cursor
    }

    /// Calculate the font-provided leading present on this line.
    fn font_leading_adjustment(&self) -> Twips {
        // Flash appears to round up the font's leading to the nearest pixel
        // and adds one. I'm not sure why.
        round_down_to_pixel(
            self.font
                .map(|f| f.get_leading_for_height(self.max_font_size))
                .unwrap_or_else(|| Twips::new(0)),
        )
    }

    /// Calculate the line-to-line leading present on ths line, including the
    /// font-leading above.
    fn line_leading_adjustment(&self) -> Twips {
        round_down_to_pixel(
            self.font
                .map(|f| f.get_leading_for_height(self.max_font_size))
                .unwrap_or_else(|| Twips::new(0))
                + Twips::from_pixels(self.current_line_span.leading),
        )
    }

    /// Apply all indents and alignment to the current line, if necessary.
    fn fixup_line(&mut self, mc: MutationContext<'gc, '_>) {
        if self.current_line.is_none() {
            return;
        }

        let mut line = self.current_line;
        let mut line_bounds = None;
        while let Some(linebox) = line {
            let mut write = linebox.write(mc);
            line = write.next_sibling();
            let (start, end, _tf, font, size, _color) = write.text_node().expect("text");

            //Flash ignores trailing spaces when aligning lines, so should we
            if self.current_line_span.align != swf::TextAlign::Left {
                write.bounds = write.bounds.with_size(Size::from(
                    font.measure(self.text[start..end].trim_end(), size),
                ));
            }

            if let Some(mut line_bounds) = line_bounds {
                line_bounds += write.bounds;
            } else {
                line_bounds = Some(write.bounds);
            }
        }

        let mut line_bounds = line_bounds.unwrap_or_else(Default::default);

        let left_adjustment =
            Self::left_alignment_offset(&self.current_line_span, self.is_first_line);
        let right_adjustment = Twips::from_pixels(self.current_line_span.right_margin);
        let align_adjustment = max(
            match self.current_line_span.align {
                swf::TextAlign::Left => Default::default(),
                swf::TextAlign::Center => {
                    (self.max_bounds - left_adjustment - right_adjustment - line_bounds.width()) / 2
                }
                swf::TextAlign::Right => {
                    self.max_bounds - left_adjustment - right_adjustment - line_bounds.width()
                }
                swf::TextAlign::Justify => {
                    log::error!("Justified text is unimplemented!");
                    Default::default()
                }
            },
            Twips::from_pixels(0.0),
        );

        let font_leading_adjustment = self.font_leading_adjustment();

        line = self.current_line;
        while let Some(linebox) = line {
            let mut write = linebox.write(mc);

            // TODO: This attempts to keep text of multiple font sizes vertically
            // aligned correctly. It does not consider the baseline of the font,
            // which is information we don't have yet.
            let font_size_adjustment = self.max_font_size - write.bounds.height();

            write.bounds += Position::from((
                left_adjustment + align_adjustment,
                font_size_adjustment + font_leading_adjustment,
            ));
            line = write.next_sibling();
        }

        line_bounds += Position::from((align_adjustment, Twips::from_pixels(0.0)));
        line_bounds += Size::from((left_adjustment + right_adjustment, font_leading_adjustment));

        self.current_line = None;

        if let Some(eb) = &mut self.exterior_bounds {
            *eb += line_bounds;
        } else {
            self.exterior_bounds = Some(line_bounds);
        }
    }

    /// Adjust the text layout cursor down to the next line.
    ///
    /// This function will also adjust any layout boxes on the current line to
    /// their correct alignment and indentation.
    fn newline(&mut self, mc: MutationContext<'gc, '_>) {
        self.fixup_line(mc);

        self.cursor.set_x(Twips::from_pixels(0.0));
        self.cursor += (
            Twips::from_pixels(0.0),
            self.max_font_size + self.line_leading_adjustment(),
        )
            .into();

        self.is_first_line = false;
    }

    /// Enter a new span.
    fn newspan(&mut self, first_span: &TextSpan) {
        if self.current_line.is_none() {
            self.current_line_span = first_span.clone();
        }

        self.max_font_size = max(self.max_font_size, Twips::from_pixels(first_span.size));
    }

    fn font(&self) -> Option<Font<'gc>> {
        self.font
    }

    fn resolve_font(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        span: &TextSpan,
    ) -> Option<Font<'gc>> {
        let library = context.library.library_for_movie_mut(movie);

        if let Some(font) = library
            .get_font_by_name(&span.font, span.bold, span.italic)
            .or_else(|| library.device_font())
        {
            self.font = Some(font);
            return self.font;
        }

        None
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
        if is_first_line {
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
        mc: MutationContext<'gc, '_>,
    ) -> (Option<GcCell<'gc, LayoutBox<'gc>>>, BoxBounds<Twips>) {
        self.fixup_line(mc);

        (
            self.first_box,
            self.exterior_bounds.unwrap_or_else(Default::default),
        )
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
    /// The text is assumed to be pulled from the same `FormatSpans`
    Text {
        /// The start position of the text to render.
        start: usize,
        end: usize,
        text_format: TextFormat,
        font: Font<'gc>,
        font_size: Collec<Twips>,
        color: Collec<swf::Color>,
    },
}

impl<'gc> LayoutBox<'gc> {
    /// Construct a text box for an HTML text node.
    pub fn from_text(
        mc: MutationContext<'gc, '_>,
        start: usize,
        end: usize,
        text_format: TextFormat,
        font: Font<'gc>,
        font_size: Twips,
        color: swf::Color,
    ) -> GcCell<'gc, Self> {
        GcCell::allocate(
            mc,
            Self {
                bounds: Default::default(),
                next_sibling: None,
                content: LayoutContent::Text {
                    start,
                    end,
                    text_format,
                    font,
                    font_size: Collec(font_size),
                    color: Collec(color),
                },
            },
        )
    }

    /// Returns the next sibling box.
    pub fn next_sibling(&self) -> Option<GcCell<'gc, LayoutBox<'gc>>> {
        self.next_sibling
    }

    ///
    pub fn append_text_fragment<'a>(
        mc: MutationContext<'gc, '_>,
        lc: &mut LayoutContext<'a, 'gc>,
        text: &'a str,
        start: usize,
        end: usize,
        span: &TextSpan,
    ) {
        let font_size = Twips::from_pixels(span.size);
        let text_size = Size::from(lc.font().unwrap().measure(text, font_size));
        let text_bounds = BoxBounds::from_position_and_size(*lc.cursor(), text_size);
        let new_text = Self::from_text(
            mc,
            start,
            end,
            span.get_text_format(),
            lc.font().unwrap(),
            font_size,
            span.color.clone(),
        );
        let mut write = new_text.write(mc);

        write.bounds = text_bounds;

        *lc.cursor_mut() += Position::from((text_size.width(), Twips::default()));
        lc.append_box(mc, new_text);
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
        let mut layout_context = LayoutContext::new(bounds, fs.text());

        for (start, _end, text, span) in fs.iter_spans() {
            if let Some(font) = layout_context.resolve_font(context, movie.clone(), &span) {
                layout_context.newspan(span);

                let font_size = Twips::from_pixels(span.size);
                let mut last_breakpoint = 0;

                if is_word_wrap {
                    let (mut width, mut offset) = layout_context.wrap_dimensions(&span);

                    while let Some(breakpoint) =
                        font.wrap_line(&text[last_breakpoint..], font_size, width, offset)
                    {
                        if breakpoint == 0 {
                            layout_context.newline(context.gc_context);

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

                        Self::append_text_fragment(
                            context.gc_context,
                            &mut layout_context,
                            &text[last_breakpoint..next_breakpoint],
                            start + last_breakpoint,
                            start + next_breakpoint,
                            span,
                        );

                        last_breakpoint = next_breakpoint;
                        if last_breakpoint >= text.len() {
                            break;
                        }

                        layout_context.newline(context.gc_context);
                        let next_dim = layout_context.wrap_dimensions(&span);

                        width = next_dim.0;
                        offset = next_dim.1;
                    }
                }

                let span_end = text.len();

                if last_breakpoint < span_end {
                    Self::append_text_fragment(
                        context.gc_context,
                        &mut layout_context,
                        &text[last_breakpoint..span_end],
                        start + last_breakpoint,
                        start + span_end,
                        span,
                    );
                }
            }
        }

        layout_context.end_layout(context.gc_context)
    }

    pub fn bounds(&self) -> BoxBounds<Twips> {
        self.bounds
    }

    /// Returns a reference to the text this box contains.
    pub fn text_node(&self) -> Option<(usize, usize, &TextFormat, Font<'gc>, Twips, swf::Color)> {
        match &self.content {
            LayoutContent::Text {
                start,
                end,
                text_format,
                font,
                font_size,
                color,
            } => Some((
                *start,
                *end,
                &text_format,
                *font,
                font_size.0,
                color.0.clone(),
            )),
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
