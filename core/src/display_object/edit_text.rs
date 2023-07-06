//! `EditText` display object and support code.

use crate::avm1::Avm1;
use crate::avm1::ExecutionReason;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier};
use crate::avm1::{
    Object as Avm1Object, StageObject as Avm1StageObject, TObject as Avm1TObject,
    Value as Avm1Value,
};
use crate::avm2::Avm2;
use crate::avm2::{
    Activation as Avm2Activation, EventObject as Avm2EventObject, Object as Avm2Object,
    StageObject as Avm2StageObject,
};
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::drawing::Drawing;
use crate::events::{ClipEvent, ClipEventResult, TextControlCode};
use crate::font::{round_down_to_pixel, Glyph, TextRenderSettings};
use crate::html::{BoxBounds, FormatSpans, LayoutBox, LayoutContent, LayoutMetrics, TextFormat};
use crate::prelude::*;
use crate::string::{utils as string_utils, AvmString, SwfStrExt as _, WStr, WString};
use crate::tag_utils::SwfMovie;
use crate::vminterface::{AvmObject, Instantiator};
use chrono::Utc;
use core::fmt;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use ruffle_render::commands::CommandHandler;
use ruffle_render::shape_utils::DrawCommand;
use ruffle_render::transform::Transform;
use std::{cell::Ref, cell::RefMut, sync::Arc};
use swf::{Color, ColorTransform, Twips};

use super::interactive::Avm2MousePick;

/// The kind of autosizing behavior an `EditText` should have, if any
#[derive(Copy, Clone, Collect, PartialEq, Eq)]
#[collect(no_drop)]
pub enum AutoSizeMode {
    None,
    Left,
    Center,
    Right,
}

/// A dynamic text field.
/// The text in this text field can be changed dynamically.
/// It may be selectable or editable by the user, depending on the text field properties.
///
/// In the Flash IDE, this is created by changing the text field type to "Dynamic".
/// In AS2, this is created using `MovieClip.createTextField`.
/// In AS3, this is created with the `TextField` class. (https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html)
///
/// (SWF19 DefineEditText pp. 171-174)
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct EditText<'gc>(GcCell<'gc, EditTextData<'gc>>);

impl fmt::Debug for EditText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EditText")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct EditTextData<'gc> {
    /// DisplayObject and InteractiveObject common properties.
    base: InteractiveObjectBase<'gc>,

    /// Static data shared among all instances of this `EditText`.
    static_data: Gc<'gc, EditTextStatic>,

    /// The underlying text format spans of the `EditText`.
    ///
    /// This is generated from HTML (with optional CSS) or set directly, and
    /// can be directly manipulated by ActionScript. It can also be raised to
    /// an equivalent HTML representation, as long as no stylesheet is present.
    ///
    /// It is lowered further into layout boxes, which are used for actual
    /// rendering.
    text_spans: FormatSpans,

    /// The color of the background fill. Only applied when has_border and has_background.
    #[collect(require_static)]
    background_color: Color,

    /// The color of the border.
    #[collect(require_static)]
    border_color: Color,

    /// The current border drawing.
    drawing: Drawing,

    /// Whether or not the width of the field should change in response to text
    /// changes, and in what direction should added or removed width should
    /// apply.
    autosize: AutoSizeMode,

    /// The calculated layout box.
    layout: Vec<LayoutBox<'gc>>,

    /// The intrinsic bounds of the laid-out text.
    intrinsic_bounds: BoxBounds<Twips>,

    /// The current intrinsic bounds of the text field.
    #[collect(require_static)]
    bounds: Rectangle<Twips>,

    /// The AVM1 object handle
    object: Option<AvmObject<'gc>>,

    /// The variable path that this text field is bound to (AVM1 only).
    variable: Option<String>,

    /// The display object that the variable binding is bound to.
    bound_stage_object: Option<Avm1StageObject<'gc>>,

    /// The selected portion of the text, or None if the text is not selected.
    selection: Option<TextSelection>,

    /// Which rendering engine this text field will use.
    render_settings: TextRenderSettings,

    /// How many pixels right the text is offset by. 0-based index.
    hscroll: f64,

    /// Information about the layout's current lines. Used by scroll properties.
    line_data: Vec<LineData>,

    /// How many lines down the text is offset by. 1-based index.
    scroll: usize,

    /// The limit of characters that can be manually input by the user.
    /// Doesn't affect script-triggered modifications.
    max_chars: i32,

    /// Flags indicating the text field's settings.
    flags: EditTextFlag,
}

// TODO: would be nicer to compute (and return) this during layout, instead of afterwards
/// Compute line (index, offset, extent) from the layout data.
fn get_line_data(layout: &[LayoutBox]) -> Vec<LineData> {
    // if there are no boxes, there are no lines
    if layout.is_empty() {
        return Vec::new();
    }

    let first_box = &layout[0];

    let mut index = 1;
    let mut offset = first_box.bounds().offset_y();
    let mut extent = first_box.bounds().extent_y();

    let mut line_data = Vec::new();

    for layout_box in layout.get(1..).unwrap() {
        let bounds = layout_box.bounds();

        // if the top of the new box is lower than the bottom of the old box, it's a new line
        if bounds.offset_y() > extent {
            // save old line and reset
            line_data.push(LineData {
                index,
                offset,
                extent,
            });

            index += 1;
            offset = bounds.offset_y();
            extent = bounds.extent_y();
        } else {
            // otherwise we continue from the previous box
            offset = offset.min(bounds.offset_y());
            extent = extent.max(bounds.extent_y());
        }
    }

    // save the final line
    line_data.push(LineData {
        index,
        offset,
        extent,
    });

    line_data
}

impl<'gc> EditText<'gc> {
    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc>,
        swf_movie: Arc<SwfMovie>,
        swf_tag: swf::EditText,
    ) -> Self {
        let default_format = TextFormat::from_swf_tag(swf_tag.clone(), swf_movie.clone(), context);
        let encoding = swf_movie.encoding();
        let text = swf_tag.initial_text().unwrap_or_default().decode(encoding);

        let mut text_spans = if swf_tag.is_html() {
            FormatSpans::from_html(&text, default_format, swf_tag.is_multiline())
        } else {
            FormatSpans::from_text(text.into_owned(), default_format)
        };

        if swf_tag.is_password() {
            text_spans.hide_text();
        }

        let autosize = if swf_tag.is_auto_size() {
            AutoSizeMode::Left
        } else {
            AutoSizeMode::None
        };

        let (layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &text_spans,
            context,
            swf_movie.clone(),
            swf_tag.bounds().width() - Twips::from_pixels(Self::INTERNAL_PADDING * 2.0),
            swf_tag.is_word_wrap(),
            !swf_tag.use_outlines(),
        );
        let line_data = get_line_data(&layout);

        let mut base = InteractiveObjectBase::default();

        base.base.matrix_mut().tx = swf_tag.bounds().x_min;
        base.base.matrix_mut().ty = swf_tag.bounds().y_min;

        let variable = if !swf_tag.variable_name().is_empty() {
            Some(swf_tag.variable_name())
        } else {
            None
        };

        // We match the flags from the DefineEditText SWF tag.
        let mut flags = EditTextFlag::from_bits_truncate(swf_tag.flags().bits());
        // For extra flags, use some of the SWF tag bits that are unused after the text field is created.
        flags &= EditTextFlag::SWF_FLAGS;
        flags.set(
            EditTextFlag::HAS_BACKGROUND,
            flags.contains(EditTextFlag::BORDER),
        );

        let et = EditText(GcCell::allocate(
            context.gc_context,
            EditTextData {
                base,
                text_spans,
                static_data: gc_arena::Gc::allocate(
                    context.gc_context,
                    EditTextStatic {
                        swf: swf_movie,
                        id: swf_tag.id(),
                        bounds: swf_tag.bounds().clone(),
                        layout: swf_tag.layout().cloned(),
                        initial_text: swf_tag
                            .initial_text()
                            .map(|s| s.decode(encoding).into_owned()),
                    },
                ),
                flags,
                background_color: Color::WHITE,
                border_color: Color::BLACK,
                drawing: Drawing::new(),
                object: None,
                layout,
                intrinsic_bounds,
                bounds: swf_tag.bounds().clone(),
                autosize,
                variable: variable.map(|s| s.to_string_lossy(encoding)),
                bound_stage_object: None,
                selection: None,
                render_settings: Default::default(),
                hscroll: 0.0,
                line_data,
                scroll: 1,
                max_chars: swf_tag.max_length().unwrap_or_default() as i32,
            },
        ));

        if swf_tag.is_auto_size() {
            et.relayout(context);
        } else {
            et.redraw_border(context.gc_context);
        }

        et
    }

    /// Create a new, dynamic `EditText`.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc>,
        swf_movie: Arc<SwfMovie>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let swf_tag = swf::EditText::new()
            .with_font_id(0, Twips::from_pixels_i32(12))
            .with_color(Some(Color::BLACK))
            .with_bounds(swf::Rectangle {
                x_min: Twips::ZERO,
                x_max: Twips::from_pixels(width),
                y_min: Twips::ZERO,
                y_max: Twips::from_pixels(height),
            })
            .with_layout(Some(Default::default()))
            .with_is_read_only(true)
            .with_is_selectable(true);
        let text_field = Self::from_swf_tag(context, swf_movie, swf_tag);

        // Set position.
        {
            let mut base = text_field.base_mut(context.gc_context);
            let matrix = base.matrix_mut();
            matrix.tx = Twips::from_pixels(x);
            matrix.ty = Twips::from_pixels(y);
        }

        text_field
    }

    pub fn text(self) -> WString {
        self.0.read().text_spans.text().into()
    }

    pub fn set_text(self, text: &WStr, context: &mut UpdateContext<'_, 'gc>) {
        let mut edit_text = self.0.write(context.gc_context);
        let default_format = edit_text.text_spans.default_format().clone();
        edit_text.text_spans = FormatSpans::from_text(text.into(), default_format);
        drop(edit_text);

        self.relayout(context);
    }

    pub fn html_text(self) -> WString {
        if self.is_html() {
            self.0.read().text_spans.to_html()
        } else {
            // Non-HTML text fields always return plain text.
            self.text()
        }
    }

    pub fn set_html_text(self, text: &WStr, context: &mut UpdateContext<'_, 'gc>) {
        if self.is_html() {
            let mut write = self.0.write(context.gc_context);
            let default_format = write.text_spans.default_format().clone();
            write.text_spans = FormatSpans::from_html(
                text,
                default_format,
                write.flags.contains(EditTextFlag::MULTILINE),
            );
            drop(write);

            self.relayout(context);
        } else {
            self.set_text(text, context);
        }
    }

    pub fn text_length(self) -> usize {
        self.0.read().text_spans.text().len()
    }

    pub fn new_text_format(self) -> TextFormat {
        self.0.read().text_spans.default_format().clone()
    }

    pub fn set_new_text_format(self, tf: TextFormat, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .text_spans
            .set_default_format(tf);
    }

    pub fn text_format(self, from: usize, to: usize) -> TextFormat {
        // TODO: Convert to byte indices
        self.0.read().text_spans.get_text_format(from, to)
    }

    pub fn set_text_format(
        self,
        from: usize,
        to: usize,
        tf: TextFormat,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        // TODO: Convert to byte indices
        self.0
            .write(context.gc_context)
            .text_spans
            .set_text_format(from, to, &tf);
        self.relayout(context);
    }

    pub fn is_editable(self) -> bool {
        !self.0.read().flags.contains(EditTextFlag::READ_ONLY)
    }

    pub fn was_static(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::WAS_STATIC)
    }

    pub fn set_editable(self, is_editable: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::READ_ONLY, !is_editable);
    }

    pub fn is_multiline(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::MULTILINE)
    }

    pub fn is_password(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::PASSWORD)
    }

    pub fn set_password(self, is_password: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::PASSWORD, is_password);
        self.relayout(context);
    }

    pub fn set_multiline(self, is_multiline: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::MULTILINE, is_multiline);
        self.relayout(context);
    }

    pub fn is_selectable(self) -> bool {
        !self.0.read().flags.contains(EditTextFlag::NO_SELECT)
    }

    pub fn set_selectable(self, is_selectable: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::NO_SELECT, !is_selectable);
    }

    pub fn is_word_wrap(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::WORD_WRAP)
    }

    pub fn set_word_wrap(self, is_word_wrap: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::WORD_WRAP, is_word_wrap);
        self.relayout(context);
    }

    pub fn autosize(self) -> AutoSizeMode {
        self.0.read().autosize
    }

    pub fn set_autosize(self, asm: AutoSizeMode, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).autosize = asm;
        self.relayout(context);
    }

    pub fn has_background(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::HAS_BACKGROUND)
    }

    pub fn set_has_background(self, gc_context: MutationContext<'gc, '_>, has_background: bool) {
        self.0
            .write(gc_context)
            .flags
            .set(EditTextFlag::HAS_BACKGROUND, has_background);
        self.redraw_border(gc_context);
    }

    pub fn background_color(self) -> Color {
        self.0.read().background_color
    }

    pub fn set_background_color(
        self,
        gc_context: MutationContext<'gc, '_>,
        background_color: Color,
    ) {
        self.0.write(gc_context).background_color = background_color;
        self.redraw_border(gc_context);
    }

    pub fn has_border(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::BORDER)
    }

    pub fn set_has_border(self, gc_context: MutationContext<'gc, '_>, has_border: bool) {
        self.0
            .write(gc_context)
            .flags
            .set(EditTextFlag::BORDER, has_border);
        self.redraw_border(gc_context);
    }

    pub fn border_color(self) -> Color {
        self.0.read().border_color
    }

    pub fn set_border_color(self, gc_context: MutationContext<'gc, '_>, border_color: Color) {
        self.0.write(gc_context).border_color = border_color;
        self.redraw_border(gc_context);
    }

    pub fn is_device_font(self) -> bool {
        !self.0.read().flags.contains(EditTextFlag::USE_OUTLINES)
    }

    pub fn set_is_device_font(self, context: &mut UpdateContext<'_, 'gc>, is_device_font: bool) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::USE_OUTLINES, !is_device_font);
        self.relayout(context);
    }

    pub fn is_html(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::HTML)
    }

    pub fn set_is_html(self, context: &mut UpdateContext<'_, 'gc>, is_html: bool) {
        self.0
            .write(context.gc_context)
            .flags
            .set(EditTextFlag::HTML, is_html);
    }

    pub fn replace_text(
        self,
        from: usize,
        to: usize,
        text: &WStr,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        self.0
            .write(context.gc_context)
            .text_spans
            .replace_text(from, to, text, None);
        self.relayout(context);
    }

    /// Construct a base text transform for a particular `EditText` span.
    ///
    /// This `text_transform` is separate from and relative to the base
    /// transform that this `EditText` automatically gets by virtue of being a
    /// `DisplayObject`.
    pub fn text_transform(self, color: Color, baseline_adjustment: Twips) -> Transform {
        let mut transform: Transform = Default::default();
        transform.color_transform.set_mult_color(&color);

        // TODO MIKE: This feels incorrect here but is necessary for correct vertical position;
        // the glyphs are rendered relative to the baseline. This should be taken into account either
        // by the layout code earlier (cursor should start at the baseline, not 0,0) and/or by
        // font.evaluate (should return transforms relative to the baseline).
        transform.matrix.ty = baseline_adjustment;

        transform
    }

    pub fn line_width(self) -> Twips {
        let edit_text = self.0.read();
        let static_data = &edit_text.static_data;

        let mut base_width = Twips::from_pixels(self.width());

        if let Some(layout) = &static_data.layout {
            base_width -= layout.left_margin;
            base_width -= layout.indent;
            base_width -= layout.right_margin;
        }

        base_width
    }

    /// Returns the variable that this text field is bound to.
    pub fn variable(&self) -> Option<Ref<str>> {
        let text = self.0.read();
        if text.variable.is_some() {
            Some(Ref::map(text, |text| text.variable.as_deref().unwrap()))
        } else {
            None
        }
    }

    pub fn set_variable(self, variable: Option<String>, activation: &mut Avm1Activation<'_, 'gc>) {
        // Clear previous binding.
        if let Some(stage_object) = self
            .0
            .write(activation.context.gc_context)
            .bound_stage_object
            .take()
        {
            stage_object.clear_text_field_binding(activation.context.gc_context, self);
        } else {
            activation
                .context
                .unbound_text_fields
                .retain(|&text_field| !DisplayObject::ptr_eq(text_field.into(), self.into()));
        }

        // Setup new binding.
        let text = self
            .0
            .read()
            .static_data
            .initial_text
            .clone()
            .unwrap_or_default();
        self.set_text(&text, &mut activation.context);

        self.0.write(activation.context.gc_context).variable = variable;
        self.try_bind_text_field_variable(activation, true);
    }

    /// Construct a base text transform for this `EditText`, to be used for
    /// evaluating fonts.
    ///
    /// The `text_transform` constitutes the base transform that all text is
    /// written into.

    /// Redraw the border of this `EditText`.
    fn redraw_border(self, gc_context: MutationContext<'gc, '_>) {
        let mut write = self.0.write(gc_context);

        write.drawing.clear();

        if write
            .flags
            .intersects(EditTextFlag::BORDER | EditTextFlag::HAS_BACKGROUND)
        {
            let line_style = write.flags.contains(EditTextFlag::BORDER).then_some(
                swf::LineStyle::new()
                    .with_width(Twips::new(1))
                    .with_color(write.border_color),
            );
            write.drawing.set_line_style(line_style);

            let fill_style = write
                .flags
                .contains(EditTextFlag::HAS_BACKGROUND)
                .then_some(swf::FillStyle::Color(write.background_color));
            write.drawing.set_fill_style(fill_style);

            let width = write.bounds.width();
            let height = write.bounds.height();
            write.drawing.draw_command(DrawCommand::MoveTo(Point::ZERO));
            write
                .drawing
                .draw_command(DrawCommand::LineTo(Point::new(Twips::ZERO, height)));
            write
                .drawing
                .draw_command(DrawCommand::LineTo(Point::new(width, height)));
            write
                .drawing
                .draw_command(DrawCommand::LineTo(Point::new(width, Twips::ZERO)));
            write.drawing.draw_command(DrawCommand::LineTo(Point::ZERO));

            drop(write);
            self.invalidate_cached_bitmap(gc_context);
        }
    }

    /// Internal padding between the bounds of the EditText and the text.
    /// Applies to each side.
    const INTERNAL_PADDING: f64 = 2.0;

    /// Relayout the `EditText`.
    ///
    /// This function operats exclusively with the text-span representation of
    /// the text, and no higher-level representation. Specifically, CSS should
    /// have already been calculated and applied to HTML trees lowered into the
    /// text-span representation.
    fn relayout(self, context: &mut UpdateContext<'_, 'gc>) {
        let mut edit_text = self.0.write(context.gc_context);
        let autosize = edit_text.autosize;
        let is_word_wrap = edit_text.flags.contains(EditTextFlag::WORD_WRAP);
        let movie = edit_text.static_data.swf.clone();
        let padding = Twips::from_pixels(EditText::INTERNAL_PADDING) * 2;

        if edit_text.flags.contains(EditTextFlag::PASSWORD) {
            // If the text is a password, hide the text
            edit_text.text_spans.hide_text();
        } else if edit_text.text_spans.has_displayed_text() {
            // If it is not a password and has displayed text, we can clear the displayed text
            edit_text.text_spans.clear_displayed_text();
        }

        let (new_layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &edit_text.text_spans,
            context,
            movie,
            edit_text.bounds.width() - padding,
            is_word_wrap,
            !edit_text.flags.contains(EditTextFlag::USE_OUTLINES),
        );

        edit_text.line_data = get_line_data(&new_layout);
        edit_text.layout = new_layout;
        edit_text.intrinsic_bounds = intrinsic_bounds;
        // reset scroll
        edit_text.hscroll = 0.0;
        edit_text.scroll = 1;

        if autosize != AutoSizeMode::None {
            if !is_word_wrap {
                // The edit text's bounds needs to have the padding baked in.
                let width = intrinsic_bounds.width() + padding;
                let new_x = match autosize {
                    AutoSizeMode::Left => edit_text.bounds.x_min,
                    AutoSizeMode::Center => {
                        (edit_text.bounds.x_min + edit_text.bounds.x_max - width) / 2
                    }
                    AutoSizeMode::Right => edit_text.bounds.x_max - width,
                    AutoSizeMode::None => unreachable!(),
                };
                edit_text.bounds.x_min = new_x;
                edit_text.bounds.set_width(width);
            } else {
                let width = edit_text.static_data.bounds.width();
                edit_text.bounds.set_width(width);
            }
            let height = intrinsic_bounds.height() + padding;
            edit_text.bounds.set_height(height);
            drop(edit_text);
            self.redraw_border(context.gc_context);
        } else {
            drop(edit_text);
            self.invalidate_cached_bitmap(context.gc_context);
        }
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, _context: &mut UpdateContext<'_, 'gc>) -> (Twips, Twips) {
        let edit_text = self.0.read();

        (
            edit_text.intrinsic_bounds.width(),
            edit_text.intrinsic_bounds.height(),
        )
    }

    /// How far the text can be scrolled right, in pixels.
    pub fn maxhscroll(self) -> f64 {
        let edit_text = self.0.read();

        // word-wrapped text can't be scrolled
        if edit_text.flags.contains(EditTextFlag::WORD_WRAP) {
            return 0.0;
        }

        let base =
            round_down_to_pixel(edit_text.intrinsic_bounds.width() - edit_text.bounds.width())
                .to_pixels()
                .max(0.0);

        // input text boxes get extra space at the end
        if !edit_text.flags.contains(EditTextFlag::READ_ONLY) {
            base + 41.0
        } else {
            base
        }
    }

    /// How many lines the text can be scrolled down
    pub fn maxscroll(self) -> usize {
        let edit_text = self.0.read();

        let line_data = &edit_text.line_data;

        if line_data.is_empty() {
            return 1;
        }

        let target = line_data.last().unwrap().extent - edit_text.bounds.height();

        // minimum line n such that n.offset > max.extent - bounds.height()
        let max_line = line_data.iter().find(|&&l| target < l.offset);
        if let Some(line) = max_line {
            line.index
        } else {
            // I don't know how this could happen, so return the limit
            line_data.last().unwrap().index
        }
    }

    /// The lowest visible line of text
    pub fn bottom_scroll(self) -> usize {
        let edit_text = self.0.read();

        let line_data = &edit_text.line_data;

        if line_data.is_empty() {
            return 1;
        }

        let scroll_offset = line_data
            .get(edit_text.scroll - 1)
            .map_or(Twips::ZERO, |l| l.offset);
        let target = edit_text.bounds.height() + scroll_offset;

        // Line before first line with extent greater than bounds.height() + line "scroll"'s offset
        let too_far = line_data.iter().find(|&&l| l.extent > target);
        if let Some(line) = too_far {
            line.index - 1
        } else {
            // all lines are visible
            line_data.last().unwrap().index
        }
    }

    /// Render a layout box, plus its children.
    fn render_layout_box(self, context: &mut RenderContext<'_, 'gc>, lbox: &LayoutBox<'gc>) {
        let origin = lbox.bounds().origin();
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(origin.x(), origin.y()),
            ..Default::default()
        });

        let edit_text = self.0.read();
        let selection = edit_text.selection;

        let caret = if let LayoutContent::Text { start, end, .. } = &lbox.content() {
            if let Some(selection) = selection {
                if selection.is_caret()
                    && !edit_text.flags.contains(EditTextFlag::READ_ONLY)
                    && selection.start() >= *start
                    && selection.end() <= *end
                    && Utc::now().timestamp_subsec_millis() / 500 == 0
                {
                    Some((selection.start() - start, end - start))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let start = if let LayoutContent::Text { start, .. } = &lbox.content() {
            *start
        } else {
            0
        };

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some((text, _tf, font, params, color)) =
            lbox.as_renderable_text(edit_text.text_spans.displayed_text())
        {
            let baseline_adjustment =
                font.get_baseline_for_height(params.height()) - params.height();
            font.evaluate(
                text,
                self.text_transform(color, baseline_adjustment),
                params,
                |pos, transform, glyph: &Glyph, advance, x| {
                    // If it's highlighted, override the color.
                    match selection {
                        Some(selection) if selection.contains(start + pos) => {
                            // Draw black selection rect
                            let selection_box = context.transform_stack.transform().matrix
                                * Matrix::create_box(
                                    advance.to_pixels() as f32,
                                    params.height().to_pixels() as f32,
                                    0.0,
                                    x + Twips::from_pixels(-1.0),
                                    Twips::from_pixels(2.0),
                                );
                            context.commands.draw_rect(Color::BLACK, selection_box);

                            // Set text color to white
                            context.transform_stack.push(&Transform {
                                matrix: transform.matrix,
                                color_transform: ColorTransform::IDENTITY,
                            });
                        }
                        _ => {
                            context.transform_stack.push(transform);
                        }
                    }

                    // Render glyph.
                    let glyph_shape_handle = glyph.shape_handle(context.renderer);
                    context
                        .commands
                        .render_shape(glyph_shape_handle, context.transform_stack.transform());
                    context.transform_stack.pop();

                    if let Some((caret_pos, length)) = caret {
                        if caret_pos == pos {
                            let caret = context.transform_stack.transform().matrix
                                * Matrix::create_box(
                                    1.0,
                                    params.height().to_pixels() as f32,
                                    0.0,
                                    x + Twips::from_pixels(-1.0),
                                    Twips::from_pixels(2.0),
                                );
                            context.commands.draw_rect(color, caret);
                        } else if pos == length - 1 && caret_pos == length {
                            let caret = context.transform_stack.transform().matrix
                                * Matrix::create_box(
                                    1.0,
                                    params.height().to_pixels() as f32,
                                    0.0,
                                    x + advance,
                                    Twips::from_pixels(2.0),
                                );
                            context.commands.draw_rect(color, caret);
                        }
                    }
                },
            );
        }

        if let Some(drawing) = lbox.as_renderable_drawing() {
            drawing.render(context);
        }

        context.transform_stack.pop();
    }

    /// Attempts to bind this text field to a property of a display object.
    /// If we find a parent display object matching the given path, we register oursevles and a property name with it.
    /// `set_text` will be called by the stage object whenever the property changes.
    /// If we don't find a display object, we register ourselves on a list of pending unbound text fields.
    /// Whenever a display object is created, the unbound list is checked to see if the new object should be bound.
    /// This is called when the text field is created, and, if the text field is in the unbound list, anytime a display object is created.
    pub fn try_bind_text_field_variable(
        self,
        activation: &mut Avm1Activation<'_, 'gc>,
        set_initial_value: bool,
    ) -> bool {
        if let Some(var_path) = self.variable() {
            let mut bound = false;

            // Any previous binding should have been cleared.
            debug_assert!(self.0.read().bound_stage_object.is_none());

            // Avoid double-borrows by copying the string.
            // TODO: Can we avoid this somehow? Maybe when we have a better string type.
            let variable_path = WString::from_utf8(&var_path);
            drop(var_path);

            let parent = self.avm1_parent().unwrap();

            activation.run_with_child_frame_for_display_object(
                "[Text Field Binding]",
                parent,
                activation.context.swf.version(),
                |activation| {
                    if let Ok(Some((object, property))) =
                        activation.resolve_variable_path(parent, &variable_path)
                    {
                        let property = AvmString::new(activation.context.gc_context, property);

                        // If this text field was just created, we immediately propagate the text to the variable (or vice versa).
                        if set_initial_value {
                            // If the property exists on the object, we overwrite the text with the property's value.
                            if object.has_property(activation, property) {
                                let value = object.get(property, activation).unwrap();
                                self.set_html_text(
                                    &value.coerce_to_string(activation).unwrap_or_default(),
                                    &mut activation.context,
                                );
                            } else {
                                // Otherwise, we initialize the property with the text field's text, if it's non-empty.
                                // Note that HTML text fields are often initialized with an empty <p> tag, which is not considered empty.
                                let text = self.text();
                                if !text.is_empty() {
                                    let _ = object.set(
                                        property,
                                        AvmString::new(activation.context.gc_context, self.text())
                                            .into(),
                                        activation,
                                    );
                                }
                            }
                        }

                        if let Some(stage_object) = object.as_stage_object() {
                            self.0
                                .write(activation.context.gc_context)
                                .bound_stage_object = Some(stage_object);
                            stage_object.register_text_field_binding(
                                activation.context.gc_context,
                                self,
                                property,
                            );
                            bound = true;
                        }
                    }
                },
            );
            bound
        } else {
            // No variable for this text field; success by default
            true
        }
    }

    /// Unsets a bound display object from this text field.
    /// Does not change the unbound text field list.
    /// Caller is responsible for adding this text field to the unbound list, if necessary.
    pub fn clear_bound_stage_object(self, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).bound_stage_object = None;
    }

    /// Propagates a text change to the bound display object.
    ///
    pub fn propagate_text_binding(self, activation: &mut Avm1Activation<'_, 'gc>) {
        if !self
            .0
            .read()
            .flags
            .contains(EditTextFlag::FIRING_VARIABLE_BINDING)
        {
            self.0.write(activation.context.gc_context).flags |=
                EditTextFlag::FIRING_VARIABLE_BINDING;
            if let Some(variable) = self.variable() {
                // Avoid double-borrows by copying the string.
                // TODO: Can we avoid this somehow? Maybe when we have a better string type.
                let variable_path = WString::from_utf8(&variable);
                drop(variable);

                if let Ok(Some((object, property))) =
                    activation.resolve_variable_path(self.avm1_parent().unwrap(), &variable_path)
                {
                    // Note that this can call virtual setters, even though the opposite direction won't work
                    // (virtual property changes do not affect the text field)
                    activation.run_with_child_frame_for_display_object(
                        "[Propagate Text Binding]",
                        self.avm1_parent().unwrap(),
                        activation.context.swf.version(),
                        |activation| {
                            let property = AvmString::new(activation.context.gc_context, property);
                            let _ = object.set(
                                property,
                                AvmString::new(activation.context.gc_context, self.html_text())
                                    .into(),
                                activation,
                            );
                        },
                    );
                }
            }
            self.0.write(activation.context.gc_context).flags -=
                EditTextFlag::FIRING_VARIABLE_BINDING;
        }
    }

    pub fn selection(self) -> Option<TextSelection> {
        self.0.read().selection
    }

    pub fn set_selection(
        self,
        selection: Option<TextSelection>,
        gc_context: MutationContext<'gc, '_>,
    ) {
        let mut text = self.0.write(gc_context);
        if let Some(mut selection) = selection {
            selection.clamp(text.text_spans.text().len());
            text.selection = Some(selection);
        } else {
            text.selection = None;
        }
    }

    pub fn render_settings(self) -> TextRenderSettings {
        self.0.read().render_settings.clone()
    }

    pub fn set_render_settings(
        self,
        gc_context: MutationContext<'gc, '_>,
        settings: TextRenderSettings,
    ) {
        self.0.write(gc_context).render_settings = settings
    }

    pub fn hscroll(self) -> f64 {
        self.0.read().hscroll
    }

    pub fn set_hscroll(self, hscroll: f64, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).hscroll = hscroll;
    }

    pub fn scroll(self) -> usize {
        self.0.read().scroll
    }

    pub fn set_scroll(self, scroll: f64, context: &mut UpdateContext<'_, 'gc>) {
        // derived experimentally. Not exact: overflows somewhere above 767100486418432.9
        // Checked in SWF 6, AVM1. Same in AVM2.
        const SCROLL_OVERFLOW_LIMIT: f64 = 767100486418433.0;
        let scroll_lines = if scroll.is_nan() || scroll < 0.0 || scroll >= SCROLL_OVERFLOW_LIMIT {
            1
        } else {
            scroll as usize
        };
        let clamped = scroll_lines.clamp(1, self.maxscroll());
        self.0.write(context.gc_context).scroll = clamped;
    }

    pub fn max_chars(self) -> i32 {
        self.0.read().max_chars
    }

    pub fn set_max_chars(self, value: i32, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).max_chars = value;
    }

    pub fn screen_position_to_index(self, position: Point<Twips>) -> Option<usize> {
        let text = self.0.read();
        let Some(mut position) = self.global_to_local(position) else {
            return None;
        };
        position.x += Twips::from_pixels(Self::INTERNAL_PADDING);
        position.y += Twips::from_pixels(Self::INTERNAL_PADDING);

        for layout_box in text.layout.iter() {
            let origin = layout_box.bounds().origin();
            let mut matrix = Matrix::translate(origin.x(), origin.y());
            matrix = matrix.inverse().expect("Invertible layout matrix");
            let local_position = matrix * position;

            if let Some((text, _tf, font, params, color)) =
                layout_box.as_renderable_text(text.text_spans.text())
            {
                let mut result = None;
                let baseline_adjustment =
                    font.get_baseline_for_height(params.height()) - params.height();
                font.evaluate(
                    text,
                    self.text_transform(color, baseline_adjustment),
                    params,
                    |pos, _transform, _glyph: &Glyph, advance, x| {
                        if local_position.x >= x
                            && local_position.x <= x + advance
                            && local_position.y >= Twips::ZERO
                            && local_position.y <= params.height()
                        {
                            if local_position.x >= x + (advance / 2) {
                                result = Some(string_utils::next_char_boundary(text, pos));
                            } else {
                                result = Some(pos);
                            }
                        }
                    },
                );
                if result.is_some() {
                    return result;
                }
            }
        }

        None
    }

    /// The number of characters that currently can be inserted, considering `TextField.maxChars`
    /// constraint, current text length, and current text selection length.
    fn available_chars(self) -> usize {
        let read = self.0.read();
        let max_chars = read.max_chars;
        if max_chars == 0 {
            usize::MAX
        } else {
            let text_len = read.text_spans.text().len() as i32;
            let selection_len = if let Some(selection) = self.selection() {
                (selection.end() - selection.start()) as i32
            } else {
                0
            };
            0.max(max_chars.max(0) - (text_len - selection_len)) as usize
        }
    }

    pub fn text_control_input(
        self,
        control_code: TextControlCode,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        if !self.is_editable() && control_code.is_edit_input() {
            return;
        }

        if let Some(selection) = self.selection() {
            let mut changed = false;
            let is_selectable = self.is_selectable();
            match control_code {
                TextControlCode::MoveLeft => {
                    let new_pos = if selection.is_caret() && selection.to > 0 {
                        string_utils::prev_char_boundary(&self.text(), selection.to)
                    } else {
                        selection.start()
                    };
                    self.set_selection(
                        Some(TextSelection::for_position(new_pos)),
                        context.gc_context,
                    );
                }
                TextControlCode::MoveRight => {
                    let new_pos = if selection.is_caret() && selection.to < self.text().len() {
                        string_utils::next_char_boundary(&self.text(), selection.to)
                    } else {
                        selection.end()
                    };
                    self.set_selection(
                        Some(TextSelection::for_position(new_pos)),
                        context.gc_context,
                    );
                }
                TextControlCode::SelectLeft => {
                    if is_selectable && selection.to > 0 {
                        let new_pos = string_utils::prev_char_boundary(&self.text(), selection.to);
                        self.set_selection(
                            Some(TextSelection::for_range(selection.from, new_pos)),
                            context.gc_context,
                        );
                    }
                }
                TextControlCode::SelectRight => {
                    if is_selectable && selection.to < self.text().len() {
                        let new_pos = string_utils::next_char_boundary(&self.text(), selection.to);
                        self.set_selection(
                            Some(TextSelection::for_range(selection.from, new_pos)),
                            context.gc_context,
                        )
                    }
                }
                TextControlCode::SelectAll => {
                    if is_selectable {
                        self.set_selection(
                            Some(TextSelection::for_range(0, self.text().len())),
                            context.gc_context,
                        );
                    }
                }
                TextControlCode::Copy => {
                    if !selection.is_caret() {
                        let text = &self.text()[selection.start()..selection.end()];
                        context.ui.set_clipboard_content(text.to_string());
                    }
                }
                TextControlCode::Paste => {
                    let text = &context.ui.clipboard_content();
                    // TODO: To match Flash Player, we should truncate pasted text that is longer than max_chars
                    // instead of canceling the paste action entirely
                    if text.len() <= self.available_chars() {
                        self.replace_text(
                            selection.start(),
                            selection.end(),
                            &WString::from_utf8(text),
                            context,
                        );
                        let new_pos = selection.start() + text.len();
                        if is_selectable {
                            self.set_selection(
                                Some(TextSelection::for_position(new_pos)),
                                context.gc_context,
                            );
                        } else {
                            self.set_selection(
                                Some(TextSelection::for_position(self.text().len())),
                                context.gc_context,
                            );
                        }
                        changed = true;
                    }
                }
                TextControlCode::Cut => {
                    if !selection.is_caret() {
                        let text = &self.text()[selection.start()..selection.end()];
                        context.ui.set_clipboard_content(text.to_string());

                        self.replace_text(
                            selection.start(),
                            selection.end(),
                            WStr::empty(),
                            context,
                        );
                        if is_selectable {
                            self.set_selection(
                                Some(TextSelection::for_position(selection.start())),
                                context.gc_context,
                            );
                        } else {
                            self.set_selection(
                                Some(TextSelection::for_position(self.text().len())),
                                context.gc_context,
                            );
                        }
                        changed = true;
                    }
                }
                TextControlCode::Backspace | TextControlCode::Delete if !selection.is_caret() => {
                    // Backspace or delete with multiple characters selected
                    self.replace_text(selection.start(), selection.end(), WStr::empty(), context);
                    self.set_selection(
                        Some(TextSelection::for_position(selection.start())),
                        context.gc_context,
                    );
                    changed = true;
                }
                TextControlCode::Backspace => {
                    // Backspace with caret
                    if selection.start() > 0 {
                        // Delete previous character
                        let text = self.text();
                        let start = string_utils::prev_char_boundary(&text, selection.start());
                        self.replace_text(start, selection.start(), WStr::empty(), context);
                        self.set_selection(
                            Some(TextSelection::for_position(start)),
                            context.gc_context,
                        );
                        changed = true;
                    }
                }
                TextControlCode::Delete => {
                    // Delete with caret
                    if selection.end() < self.text_length() {
                        // Delete next character
                        let text = self.text();
                        let end = string_utils::next_char_boundary(&text, selection.start());
                        self.replace_text(selection.start(), end, WStr::empty(), context);
                        // No need to change selection
                        changed = true;
                    }
                }
                _ => {}
            }
            if changed {
                let mut activation = Avm1Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Propagate Text Binding]"),
                    self.into(),
                );
                self.propagate_text_binding(&mut activation);
                self.on_changed(&mut activation);
            }
        }
    }

    pub fn text_input(self, character: char, context: &mut UpdateContext<'_, 'gc>) {
        if self.0.read().flags.contains(EditTextFlag::READ_ONLY) {
            return;
        }

        if let Some(selection) = self.selection() {
            let mut changed = false;
            match character as u8 {
                code if !(code as char).is_control() => {
                    if self.available_chars() > 0 {
                        self.replace_text(
                            selection.start(),
                            selection.end(),
                            &WString::from_char(character),
                            context,
                        );
                        let new_pos = selection.start() + character.len_utf8();
                        self.set_selection(
                            Some(TextSelection::for_position(new_pos)),
                            context.gc_context,
                        );
                        changed = true;
                    }
                }
                _ => {}
            }

            if changed {
                let mut activation = Avm1Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Propagate Text Binding]"),
                    self.into(),
                );
                self.propagate_text_binding(&mut activation);
                self.on_changed(&mut activation);
            }
        }
    }

    fn initialize_as_broadcaster(&self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Avm1Value::Object(object) = self.object() {
            activation.context.avm1.broadcaster_functions().initialize(
                activation.context.gc_context,
                object,
                activation.context.avm1.prototypes().array,
            );

            if let Ok(Avm1Value::Object(listeners)) = object.get("_listeners", activation) {
                let length = listeners.length(activation);
                if matches!(length, Ok(0)) {
                    // Add the TextField as its own listener to match Flash's behavior
                    // This makes it so that the TextField's handlers are called before other listeners'.
                    listeners.set_element(activation, 0, object.into()).unwrap();
                } else {
                    tracing::warn!("_listeners should be empty");
                }
            }
        }
    }

    fn on_changed(&self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Avm1Value::Object(object) = self.object() {
            let _ = object.call_method(
                "broadcastMessage".into(),
                &["onChanged".into(), object.into()],
                activation,
                ExecutionReason::Special,
            );
        } else if let Avm2Value::Object(object) = self.object2() {
            let change_evt = Avm2EventObject::bare_event(
                &mut activation.context,
                "change",
                true,  /* bubbles */
                false, /* cancelable */
            );
            Avm2::dispatch_event(&mut activation.context, change_evt, object);
        }
    }

    /// Construct the text field's AVM1 representation.
    fn construct_as_avm1_object(&self, context: &mut UpdateContext<'_, 'gc>, run_frame: bool) {
        let mut text = self.0.write(context.gc_context);
        if text.object.is_none() {
            let object: Avm1Object<'gc> = Avm1StageObject::for_display_object(
                context.gc_context,
                (*self).into(),
                context.avm1.prototypes().text_field,
            )
            .into();

            text.object = Some(object.into());
        }
        drop(text);

        Avm1::run_with_stack_frame_for_display_object((*self).into(), context, |activation| {
            // If this text field has a variable set, initialize text field binding.
            if !self.try_bind_text_field_variable(activation, true) {
                activation.context.unbound_text_fields.push(*self);
            }
            // People can bind to properties of TextFields the same as other display objects.
            self.bind_text_field_variables(activation);

            self.initialize_as_broadcaster(activation);
        });

        if run_frame {
            self.run_frame_avm1(context);
        }
    }

    /// Construct the text field's AVM2 representation.
    fn construct_as_avm2_object(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        display_object: DisplayObject<'gc>,
    ) {
        let textfield_constr = context.avm2.classes().textfield;
        let mut activation = Avm2Activation::from_nothing(context.reborrow());

        match Avm2StageObject::for_display_object_childless(
            &mut activation,
            display_object,
            textfield_constr,
        ) {
            Ok(object) => {
                let object: Avm2Object<'gc> = object.into();
                self.0.write(activation.context.gc_context).object = Some(object.into())
            }
            Err(e) => tracing::error!(
                "Got {} when constructing AVM2 side of dynamic text field",
                e
            ),
        }
    }

    /// Count the number of lines in the text box's layout.
    pub fn layout_lines(self) -> usize {
        self.0.read().line_data.len()
    }

    /// Calculate the layout metrics for a given line.
    ///
    /// Returns None if the line does not exist or there is not enough data
    /// about the line to calculate metrics with.
    pub fn layout_metrics(self, line: Option<usize>) -> Option<LayoutMetrics> {
        let line = line.and_then(|line| self.0.read().line_data.get(line).copied());
        let mut union_bounds = None;
        let mut font = None;
        let mut text_format = None;

        let read = self.0.read();

        for layout_box in read.layout.iter() {
            if let Some(line) = line {
                if layout_box.bounds().offset_y() < line.offset
                    || layout_box.bounds().extent_y() > line.extent
                {
                    continue;
                }
            }

            if let Some(bounds) = &mut union_bounds {
                *bounds += layout_box.bounds();
            } else {
                union_bounds = Some(layout_box.bounds());
            }

            if font.is_none() {
                match layout_box.content() {
                    LayoutContent::Text {
                        font: box_font,
                        text_format: box_text_format,
                        ..
                    } => {
                        font = Some(box_font);
                        text_format = Some(box_text_format);
                    }
                    LayoutContent::Bullet {
                        font: box_font,
                        text_format: box_text_format,
                        ..
                    } => {
                        font = Some(box_font);
                        text_format = Some(box_text_format);
                    }
                    LayoutContent::Drawing { .. } => {}
                }
            }
        }

        let union_bounds = union_bounds?;
        let font = font?;
        let size = Twips::from_pixels(text_format?.size?);
        let ascent = font.get_baseline_for_height(size);
        let descent = font.get_descent_for_height(size);
        let leading = Twips::from_pixels(text_format?.leading?);

        Some(LayoutMetrics {
            ascent,
            descent,
            leading,
            width: union_bounds.width(),
            height: union_bounds.height() + descent + leading,
            x: union_bounds.offset_x() + Twips::from_pixels(EditText::INTERNAL_PADDING),
        })
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().static_data.swf.clone()
    }

    /// Construct objects placed on this frame.
    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc>) {
        if context.is_action_script_3() && matches!(self.object2(), Avm2Value::Null) {
            self.construct_as_avm2_object(context, (*self).into());
            self.on_construction_complete(context);
        }
    }

    fn run_frame_avm1(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        Some(*self)
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if !context.is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        if !self.movie().is_action_script_3() {
            self.construct_as_avm1_object(context, run_frame);
        }
    }

    fn object(&self) -> Avm1Value<'gc> {
        self.0
            .read()
            .object
            .and_then(|o| o.as_avm1_object())
            .map(Avm1Value::from)
            .unwrap_or(Avm1Value::Undefined)
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .object
            .and_then(|o| o.as_avm2_object())
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'_, 'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).object = Some(to.into());
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        self.0.read().bounds.clone()
    }

    // The returned position x and y of a text field is offset by the text bounds.
    fn x(&self) -> Twips {
        let edit_text = self.0.read();
        let offset = edit_text.bounds.x_min;
        edit_text.base.base.x() + offset
    }

    fn set_x(&self, gc_context: MutationContext<'gc, '_>, x: Twips) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.x_min;
        edit_text.base.base.set_x(x - offset);
        drop(edit_text);
        self.redraw_border(gc_context);
    }

    fn y(&self) -> Twips {
        let edit_text = self.0.read();
        let offset = edit_text.bounds.y_min;
        edit_text.base.base.y() + offset
    }

    fn set_y(&self, gc_context: MutationContext<'gc, '_>, y: Twips) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.y_min;
        edit_text.base.base.set_y(y - offset);
        drop(edit_text);
        self.redraw_border(gc_context);
    }

    fn width(&self) -> f64 {
        let edit_text = self.0.read();
        (edit_text.base.base.transform.matrix * edit_text.bounds.clone())
            .width()
            .to_pixels()
    }

    fn set_width(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_width(Twips::from_pixels(value));
        write.base.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn height(&self) -> f64 {
        let edit_text = self.0.read();
        (edit_text.base.base.transform.matrix * edit_text.bounds.clone())
            .height()
            .to_pixels()
    }

    fn set_height(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_height(Twips::from_pixels(value));
        write.base.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn set_matrix(&self, gc_context: MutationContext<'gc, '_>, matrix: Matrix) {
        self.0.write(gc_context).base.base.set_matrix(matrix);
        self.redraw_border(gc_context);
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        let edit_text = self.0.read();
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(edit_text.bounds.x_min, edit_text.bounds.y_min),
            ..Default::default()
        });

        edit_text.drawing.render(context);

        context.commands.push_mask();
        let mask = Matrix::create_box(
            edit_text.bounds.width().to_pixels() as f32,
            edit_text.bounds.height().to_pixels() as f32,
            0.0,
            Twips::ZERO,
            Twips::ZERO,
        );
        context.commands.draw_rect(
            Color::WHITE,
            context.transform_stack.transform().matrix * mask,
        );
        context.commands.activate_mask();

        let scroll_offset = if edit_text.scroll > 1 {
            let line_data = &edit_text.line_data;

            if let Some(line_data) = line_data.get(edit_text.scroll - 1) {
                line_data.offset
            } else {
                Twips::ZERO
            }
        } else {
            Twips::ZERO
        };
        // TODO: Where does this come from? How is this different than INTERNAL_PADDING? Does this apply to y as well?
        // If this is actually right, offset the border in `redraw_border` instead of doing an extra push.
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(
                Twips::from_pixels(Self::INTERNAL_PADDING) - Twips::from_pixels(edit_text.hscroll),
                Twips::from_pixels(Self::INTERNAL_PADDING) - scroll_offset,
            ),
            ..Default::default()
        });

        if edit_text.layout.is_empty() && !edit_text.flags.contains(EditTextFlag::READ_ONLY) {
            let selection = edit_text.selection;
            if let Some(selection) = selection {
                if selection.is_caret()
                    && selection.start() == 0
                    && Utc::now().timestamp_subsec_millis() / 500 == 0
                {
                    let caret = context.transform_stack.transform().matrix
                        * Matrix::create_box(
                            1.0,
                            edit_text
                                .text_spans
                                .default_format()
                                .size
                                .unwrap_or_default() as f32,
                            0.0,
                            Twips::from_pixels(-1.0),
                            Twips::from_pixels(2.0),
                        );
                    context.commands.draw_rect(Color::BLACK, caret);
                }
            }
        } else {
            for layout_box in edit_text.layout.iter() {
                self.render_layout_box(context, layout_box);
            }
        }

        context.transform_stack.pop();

        context.commands.deactivate_mask();
        context.commands.draw_rect(
            Color::WHITE,
            context.transform_stack.transform().matrix * mask,
        );
        context.commands.pop_mask();

        context.transform_stack.pop();
    }

    fn allow_as_mask(&self) -> bool {
        false
    }

    fn avm1_unload(&self, context: &mut UpdateContext<'_, 'gc>) {
        let had_focus = self.0.read().flags.contains(EditTextFlag::HAS_FOCUS);
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }

        // Unbind any display objects bound to this text.
        if let Some(stage_object) = self.0.write(context.gc_context).bound_stage_object.take() {
            stage_object.clear_text_field_binding(context.gc_context, *self);
        }

        // Unregister any text fields that may be bound to *this* text field.
        if let Avm1Value::Object(object) = self.object() {
            if let Some(stage_object) = object.as_stage_object() {
                stage_object.unregister_text_field_bindings(context);
            }
        }
        if self.variable().is_some() {
            context
                .unbound_text_fields
                .retain(|&text_field| !DisplayObject::ptr_eq(text_field.into(), (*self).into()));
        }

        self.set_avm1_removed(context.gc_context, true);
    }

    fn on_focus_changed(&self, gc_context: MutationContext<'gc, '_>, focused: bool) {
        let mut text = self.0.write(gc_context);
        text.flags.set(EditTextFlag::HAS_FOCUS, focused);
        if !focused {
            text.selection = None;
        }
    }

    fn is_focusable(&self, _context: &mut UpdateContext<'_, 'gc>) -> bool {
        // Even if this isn't selectable or editable, a script can focus on it manually.
        true
    }
}

impl<'gc> TInteractiveObject<'gc> for EditText<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn raw_interactive_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'_, 'gc>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if event != ClipEvent::Press {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if self.is_editable() || self.is_selectable() {
            let tracker = context.focus_tracker;
            tracker.set(Some(self.into()), context);
        }
        if let Some(position) = self
            .screen_position_to_index(*context.mouse_position)
            .map(TextSelection::for_position)
        {
            self.0.write(context.gc_context).selection = Some(position);
        } else {
            self.0.write(context.gc_context).selection =
                Some(TextSelection::for_position(self.text_length()));
        }

        ClipEventResult::Handled
    }

    fn mouse_pick_avm1(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // The text is hovered if the mouse is over any child nodes.
        if self.visible()
            && self.mouse_enabled()
            && self.is_selectable()
            && self.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK)
        {
            Some((*self).into())
        } else {
            None
        }
    }

    fn mouse_pick_avm2(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        // The text is hovered if the mouse is over any child nodes.
        if self.visible() && self.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
            // Note - for mouse-enabled selectable text, we consider this to be a hit (which
            // will cause us to show the proper cursor on mouse over).
            // However, in `Interactive::event_dispatch_to_avm2`, we will prevent mouse events
            // from being fired at all if the text is selectable and 'was_static()'.
            if self.mouse_enabled() && (self.is_selectable() || !self.was_static()) {
                Avm2MousePick::Hit((*self).into())
            } else {
                Avm2MousePick::PropagateToParent
            }
        } else {
            Avm2MousePick::Miss
        }
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'_, 'gc>) -> MouseCursor {
        if self.is_selectable() {
            MouseCursor::IBeam
        } else {
            MouseCursor::Arrow
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Collect)]
    #[collect(require_static)]
    struct EditTextFlag: u16 {
        const FIRING_VARIABLE_BINDING = 1 << 0;
        const HAS_BACKGROUND = 1 << 1;
        const HAS_FOCUS = 1 << 2;

        // The following bits need to match `swf::EditTextFlag`.
        const READ_ONLY = 1 << 3;
        const PASSWORD = 1 << 4;
        const MULTILINE = 1 << 5;
        const WORD_WRAP = 1 << 6;
        const USE_OUTLINES = 1 << 8;
        const HTML = 1 << 9;
        const WAS_STATIC = 1 << 10;
        const BORDER = 1 << 11;
        const NO_SELECT = 1 << 12;
        const SWF_FLAGS = Self::READ_ONLY.bits() | Self::PASSWORD.bits() | Self::MULTILINE.bits() | Self::WORD_WRAP.bits() | Self::USE_OUTLINES.bits() |
                          Self::HTML.bits() | Self::WAS_STATIC.bits() | Self::BORDER.bits() | Self::NO_SELECT.bits();
    }
}

/// Static data shared between all instances of a text object.
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
struct EditTextStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    bounds: swf::Rectangle<Twips>,
    layout: Option<swf::TextLayout>,
    initial_text: Option<WString>,
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(require_static)]
pub struct TextSelection {
    from: usize,
    to: usize,
}

/// Information about the start and end y-coordinates of a given line of text
#[derive(Copy, Clone, Debug, Collect)]
#[collect(require_static)]
pub struct LineData {
    index: usize,
    /// How many twips down the highest point of the line is
    offset: Twips,
    /// How many twips down the lowest point of the line is
    extent: Twips,
}

impl TextSelection {
    pub fn for_position(position: usize) -> Self {
        Self {
            from: position,
            to: position,
        }
    }

    pub fn for_range(from: usize, to: usize) -> Self {
        Self { from, to }
    }

    /// The "from" part of the range is where the user started the selection.
    /// It may be greater than "to", for example if the user dragged a selection box from right to
    /// left.
    pub fn from(&self) -> usize {
        self.from
    }

    /// The "to" part of the range is where the user ended the selection.
    /// This also may be called the caret position - it is the last place the user placed the
    /// caret and any text or changes to the range will be done by this position.
    /// It may be less than "from", for example if the user dragged a selection box from right to
    /// left.
    pub fn to(&self) -> usize {
        self.to
    }

    /// The "start" part of the range is the smallest (closest to 0) part of this selection range.
    pub fn start(&self) -> usize {
        self.from.min(self.to)
    }

    /// The "end" part of the range is the largest (farthest from 0) part of this selection range.
    pub fn end(&self) -> usize {
        self.from.max(self.to)
    }

    /// Clamps this selection to the maximum length provided.
    /// Neither from nor to will be greater than this length.
    pub fn clamp(&mut self, length: usize) {
        if self.from > length {
            self.from = length;
        }
        if self.to > length {
            self.to = length;
        }
    }

    /// Checks whether the given position falls within the range of this selection
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start() && pos < self.end()
    }

    /// Returns true if this selection is a singular caret within the text,
    /// as opposed to multiple characters.
    /// If this is true, text is inserted at the position.
    /// If this is false, text is replaced at the positions.
    pub fn is_caret(&self) -> bool {
        self.to == self.from
    }
}
