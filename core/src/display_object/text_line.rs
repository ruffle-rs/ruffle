//! The TextLine display object, backing flash.text.engine.TextLine.

use crate::avm1::Object as Avm1Object;
use crate::avm2::StageObject as Avm2StageObject;
use crate::avm2::object::TextBlockObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObject, DisplayObjectBase, EditText, HitTestOptions,
    InteractiveObject, TDisplayObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::font::FontLike;
use crate::fte::{TextBaselineValue, TextLineValidity};
use crate::html::LayoutContent;
use crate::string::WStr;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::matrix::Matrix;
use ruffle_render::transform::Transform;
use std::cell::Cell;
use swf::{CharacterId, Point, Rectangle, Twips};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextLine<'gc>(Gc<'gc, TextLineData<'gc>>);

impl fmt::Debug for TextLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextLine")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextLineData<'gc> {
    base: InteractiveObjectBase<'gc>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    fallback: EditText<'gc>,
    movie: SwfMovie<'gc>,

    validity: Lock<TextLineValidity<'gc>>,

    text_block: Lock<Option<TextBlockObject<'gc>>>,
    hide_block_from_script: Cell<bool>,

    specified_width: Cell<f64>,

    /// The `baselineZero` of the block when this line was created. Flash keeps
    /// using it after the line is released from its block.
    baseline_zero: Cell<TextBaselineValue>,

    raw_text_length: Cell<u32>,

    begin_index: Cell<u32>,
    end_index: Cell<u32>,
    line_index: Cell<u32>,

    previous_line: Lock<Option<TextLine<'gc>>>,
    next_line: Lock<Option<TextLine<'gc>>>,
}

impl<'gc> TextLine<'gc> {
    pub fn new(
        context: &mut UpdateContext<'gc>,
        movie: SwfMovie<'gc>,
        fallback: EditText<'gc>,
    ) -> Self {
        TextLine(Gc::new(
            context.gc(),
            TextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                fallback,
                movie,
                validity: Lock::new(TextLineValidity::Valid),
                text_block: Lock::new(None),
                hide_block_from_script: Cell::new(false),
                specified_width: Cell::new(0.0),
                baseline_zero: Cell::new(TextBaselineValue::Roman),
                raw_text_length: Cell::new(0),
                begin_index: Cell::new(0),
                end_index: Cell::new(0),
                line_index: Cell::new(0),
                previous_line: Lock::new(None),
                next_line: Lock::new(None),
            },
        ))
    }

    pub fn reset_properties(self, mc: &Mutation<'gc>) {
        // TODO: Reset more properties

        // Reset display object properties
        self.set_x(Twips::ZERO);
        self.set_y(Twips::ZERO);

        // Reset text line properties
        self.set_validity(TextLineValidity::Valid, mc);
        self.set_text_block(None, mc);
        self.set_hide_block_from_script(false);

        self.set_specified_width(0.0);
        self.set_baseline_zero(TextBaselineValue::Roman);
        self.set_raw_text_length(0);
        self.set_begin_index(0);
        self.set_end_index(0);
        self.set_line_index(0);

        self.set_previous_line(None, mc);
        self.set_next_line(None, mc);
    }

    /// Release this line from its siblings and the block it's in.
    ///
    /// Doing this will set the validity of the line and all its successors to
    /// "invalid".
    pub fn release(self, mc: &Mutation<'gc>) {
        let block = self.text_block().expect("Line is in a text block");

        let block_first_line = block.first_line().expect("Text block has lines");

        let previous_line = self.previous_line();
        let next_line = self.next_line();

        // If this line was the text block's first line, set it to the next line
        if DisplayObject::ptr_eq(self, block_first_line) {
            block.set_first_line(next_line, mc);
        }

        // This line is, obviously, invalid now
        self.set_validity(TextLineValidity::Invalid, mc);

        // Successors of this line also become invalid, as their predecessor is invalid
        for line in self.next_lines() {
            line.set_validity(TextLineValidity::Invalid, mc);
        }

        // Make the doubly-linked list of lines skip over this one
        if let Some(previous_line) = previous_line {
            previous_line.set_next_line(next_line, mc);
        }
        if let Some(next_line) = next_line {
            next_line.set_previous_line(previous_line, mc);
        }

        // Finally, completely disconnect this line from its siblings and the block
        self.set_text_block(None, mc);
        self.set_previous_line(None, mc);
        self.set_next_line(None, mc);
    }

    pub fn measure_text(self, context: &mut UpdateContext<'gc>) -> (Twips, Twips) {
        self.0.fallback.measure_text(context)
    }

    pub fn has_tabs(self) -> bool {
        self.0.fallback.text().contains(b'\t')
    }

    /// Returns the laid-out line's `(ascent, descent)`.
    pub fn metrics(self) -> (f64, f64) {
        let layout = self.0.fallback.layout();
        let Some(line) = layout.lines().first() else {
            debug_assert!(false, "Should not be reachable");
            return Default::default();
        };

        (line.ascent().to_pixels(), line.descent().to_pixels())
    }

    pub fn fallback(self) -> EditText<'gc> {
        self.0.fallback
    }

    /// The number of atoms in the line, taking each character as an atom,
    /// except for CRLF, which is a single atom.
    // TODO: Ligatures aren't handled; Flash makes each of them a single atom.
    pub fn atom_count(self) -> u32 {
        self.0.fallback.text().len() as u32
    }

    pub fn baseline_zero(self) -> TextBaselineValue {
        self.0.baseline_zero.get()
    }

    pub fn set_baseline_zero(self, value: TextBaselineValue) {
        self.0.baseline_zero.set(value);
    }

    /// The advance of a space in the line's font, which Flash uses as the
    /// width of the line terminator's atom.
    pub fn space_advance(self) -> Twips {
        let layout = self.0.fallback.layout();
        let text_box = layout.lines().first().and_then(|line| {
            line.boxes_iter()
                .find_map(|layout_box| match layout_box.content() {
                    LayoutContent::Text {
                        font_set, params, ..
                    } => Some((*font_set, *params)),
                    _ => None,
                })
        });

        let mut advance = Twips::ZERO;

        if let Some((font_set, params)) = text_box {
            font_set.evaluate(WStr::from_units(b" "), params, |_, _, _, space, _| {
                advance = space;
            });
        }

        advance
    }

    /// The fallback text field lays out its text below its top edge, inset by
    /// its gutter, while a `TextLine`'s origin is the start of its baseline.
    fn fallback_to_local_matrix(self) -> Matrix {
        let ascent = self
            .0
            .fallback
            .layout()
            .lines()
            .first()
            .map(|line| line.ascent())
            .unwrap_or_default();

        Matrix::translate(-EditText::GUTTER, -(EditText::GUTTER + ascent))
    }

    pub fn validity(self) -> TextLineValidity<'gc> {
        self.0.validity.get()
    }

    pub fn set_validity(self, validity: TextLineValidity<'gc>, mc: &Mutation<'gc>) {
        if matches!(validity, TextLineValidity::Static) {
            // NOTE: The text line is not disconnected from its sibling lines,
            // nor is it truly disconnected from its owner block. However,
            // attempting to access the block using `line.textBlock` in AS
            // always returns `null`, even if the validity of this line is later
            // set back to some other value.
            self.set_hide_block_from_script(true);
        }

        unlock!(Gc::write(mc, self.0), TextLineData, validity).set(validity);
    }

    pub fn text_block(self) -> Option<TextBlockObject<'gc>> {
        self.0.text_block.get()
    }

    pub fn text_block_from_script(self) -> Option<TextBlockObject<'gc>> {
        if self.0.hide_block_from_script.get() {
            None
        } else {
            self.0.text_block.get()
        }
    }

    pub fn set_text_block(self, text_block: Option<TextBlockObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextLineData, text_block).set(text_block);
    }

    pub fn set_hide_block_from_script(self, value: bool) {
        self.0.hide_block_from_script.set(value);
    }

    pub fn specified_width(self) -> f64 {
        self.0.specified_width.get()
    }

    pub fn set_specified_width(self, value: f64) {
        self.0.specified_width.set(value);
    }

    pub fn raw_text_length(self) -> u32 {
        self.0.raw_text_length.get()
    }

    pub fn set_raw_text_length(self, value: u32) {
        self.0.raw_text_length.set(value);
    }

    pub fn begin_index(self) -> u32 {
        self.0.begin_index.get()
    }

    pub fn set_begin_index(self, value: u32) {
        self.0.begin_index.set(value);
    }

    pub fn end_index(self) -> u32 {
        self.0.end_index.get()
    }

    pub fn set_end_index(self, value: u32) {
        self.0.end_index.set(value);
    }

    pub fn line_index(self) -> u32 {
        self.0.line_index.get()
    }

    pub fn set_line_index(self, value: u32) {
        self.0.line_index.set(value);
    }

    pub fn previous_line(self) -> Option<TextLine<'gc>> {
        self.0.previous_line.get()
    }

    pub fn set_previous_line(self, value: Option<TextLine<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextLineData, previous_line).set(value);
    }

    pub fn next_line(self) -> Option<TextLine<'gc>> {
        self.0.next_line.get()
    }

    pub fn set_next_line(self, value: Option<TextLine<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextLineData, next_line).set(value);
    }

    pub fn next_lines(self) -> impl Iterator<Item = TextLine<'gc>> {
        core::iter::successors(self.next_line(), |line| line.next_line())
    }

    pub fn previous_lines(self) -> impl Iterator<Item = TextLine<'gc>> {
        core::iter::successors(self.previous_line(), |line| line.previous_line())
    }
}

impl<'gc> TDisplayObject<'gc> for TextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn id(self) -> CharacterId {
        0
    }

    fn movie(self) -> SwfMovie<'gc> {
        self.0.movie
    }

    fn replace_with(self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {}

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&Transform {
            matrix: self.fallback_to_local_matrix(),
            ..Default::default()
        });
        self.0.fallback.render_self(context);
        context.transform_stack.pop();
    }

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        let layout = self.0.fallback.layout();
        let Some(line) = layout.lines().first() else {
            return Rectangle::default();
        };

        Rectangle {
            x_min: Twips::ZERO,
            y_min: -line.ascent(),
            x_max: layout.text_size().width(),
            // Unlike Flash, Ruffle doesn't break lines to fit the specified
            // width, so the fallback text field may wrap into more rows.
            y_max: layout.text_size().height() - line.ascent(),
        }
    }

    fn hit_test_shape(
        self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if options.contains(HitTestOptions::SKIP_INVISIBLE) && !self.visible() {
            return false;
        }
        self.world_bounds(BoundsMode::Engine).contains(point)
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), TextLineData, avm2_object).set(Some(to));
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        self.set_default_instance_name(context);
    }
}

impl<'gc> TInteractiveObject<'gc> for TextLine<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn mouse_pick_avm2(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'gc>) -> MouseCursor {
        MouseCursor::Arrow
    }
}
