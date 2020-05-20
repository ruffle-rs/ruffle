//! Layout box structure

use crate::context::UpdateContext;
use crate::font::Font;
use crate::html::dimensions::{BoxBounds, Position, Size};
use crate::html::text_format::{FormatSpans, TextFormat, TextSpan};
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, GcCell, MutationContext};
use std::sync::Arc;
use swf::Twips;

/// Contains information relating to the current layout operation.
pub struct LayoutContext<'gc> {
    /// The position to put text into.
    cursor: Position<Twips>,

    /// The resolved font object to use when measuring text.
    font: Option<Font<'gc>>,

    /// The start of the current chain of layout boxes.
    first_box: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The end of the current chain of layout boxes.
    last_box: Option<GcCell<'gc, LayoutBox<'gc>>>,
}

impl<'gc> Default for LayoutContext<'gc> {
    fn default() -> Self {
        Self {
            cursor: Default::default(),
            font: None,
            first_box: None,
            last_box: None,
        }
    }
}

impl<'gc> LayoutContext<'gc> {
    fn cursor(&self) -> &Position<Twips> {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut Position<Twips> {
        &mut self.cursor
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

    fn append_box(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        to_append: GcCell<'gc, LayoutBox<'gc>>,
    ) {
        if self.first_box.is_none() {
            self.first_box = Some(to_append);
        }

        if let Some(last) = self.last_box {
            last.write(gc_context).next_sibling = Some(to_append);
        }

        self.last_box = Some(to_append);
    }

    fn end_layout(self) -> Option<GcCell<'gc, LayoutBox<'gc>>> {
        self.first_box
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
    pub fn append_text_fragment(
        mc: MutationContext<'gc, '_>,
        lc: &mut LayoutContext<'gc>,
        text: &str,
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
    pub fn lower_from_text_spans(
        fs: &FormatSpans,
        context: &mut UpdateContext<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
        bounds: Twips,
    ) -> Option<GcCell<'gc, LayoutBox<'gc>>> {
        let mut layout_context: LayoutContext = Default::default();

        for (start, _end, text, span) in fs.iter_spans() {
            if let Some(font) = layout_context.resolve_font(context, movie.clone(), &span) {
                let font_size = Twips::from_pixels(span.size);
                let breakpoint_list =
                    font.split_wrapped_lines(&text, font_size, bounds, layout_context.cursor().x());

                let end = text.len();

                let mut last_breakpoint = 0;

                for breakpoint in breakpoint_list {
                    if last_breakpoint != breakpoint {
                        Self::append_text_fragment(
                            context.gc_context,
                            &mut layout_context,
                            &text[last_breakpoint..breakpoint],
                            start + last_breakpoint,
                            start + breakpoint,
                            span,
                        );
                    }

                    last_breakpoint = breakpoint;
                }

                Self::append_text_fragment(
                    context.gc_context,
                    &mut layout_context,
                    &text[last_breakpoint..end],
                    start + last_breakpoint,
                    start + end,
                    span,
                );
            }
        }

        layout_context.end_layout()
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
