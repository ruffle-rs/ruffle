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
    StageObject as Avm2StageObject, TObject as _,
};
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::events::{ClipEvent, ClipEventResult, TextControlCode};
use crate::font::{round_down_to_pixel, FontType, Glyph, TextRenderSettings};
use crate::html;
use crate::html::{
    FormatSpans, Layout, LayoutBox, LayoutContent, LayoutMetrics, Position, TextFormat,
};
use crate::prelude::*;
use crate::string::{utils as string_utils, AvmString, SwfStrExt as _, WStr, WString};
use crate::tag_utils::SwfMovie;
use crate::vminterface::{AvmObject, Instantiator};
use chrono::DateTime;
use chrono::Utc;
use core::fmt;
use either::Either;
use gc_arena::{Collect, Gc, GcCell, Mutation};
use ruffle_render::commands::CommandHandler;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use ruffle_wstr::WStrToUtf8;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::{cell::Ref, cell::RefMut, sync::Arc};
use swf::ColorTransform;
use unic_segment::WordBoundIndices;

use super::interactive::Avm2MousePick;

/// The kind of autosizing behavior an `EditText` should have, if any
#[derive(Copy, Clone, Collect, Debug, PartialEq, Eq)]
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
    #[collect(require_static)]
    text_spans: FormatSpans,

    /// The color of the background fill. Only applied when has_border and has_background.
    #[collect(require_static)]
    background_color: Color,

    /// The color of the border.
    #[collect(require_static)]
    border_color: Color,

    /// Whether the width of the field should change in response to text
    /// changes, and in what direction the added or removed width should
    /// apply.
    autosize: AutoSizeMode,

    // Values set by set_width and set_height.
    #[collect(require_static)]
    requested_width: Twips,

    #[collect(require_static)]
    requested_height: Twips,

    /// The calculated layout.
    layout: Layout<'gc>,

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
    /// Note: Selections work differently in AVM1, AVM2, and Ruffle.
    ///
    /// In AVM1, there is one global optional selection. If present, it applies to whatever text field is focused.
    /// In AVM2, every text field has its own mandatory selection.
    /// In Ruffle, every text field has its own optional selection. This hybrid approach means manually maintaining
    /// the invariants that selection is always None for an unfocused AVM1 field, and never None for an AVM2 field.
    #[collect(require_static)]
    selection: Option<TextSelection>,

    /// Which rendering engine this text field will use.
    #[collect(require_static)]
    render_settings: TextRenderSettings,

    /// How many pixels right the text is offset by. 0-based index.
    hscroll: f64,

    /// How many lines down the text is offset by. 1-based index.
    scroll: usize,

    /// The limit of characters that can be manually input by the user.
    /// Doesn't affect script-triggered modifications.
    max_chars: i32,

    /// Indicates if the text is scrollable using the mouse wheel.
    mouse_wheel_enabled: bool,

    /// Flags indicating the text field's settings.
    #[collect(require_static)]
    flags: EditTextFlag,

    /// Whether this EditText represents an AVM2 TextLine.
    is_tlf: bool,

    /// Restrict what characters the user may input.
    #[collect(require_static)]
    restrict: EditTextRestrict,
}

impl<'gc> EditTextData<'gc> {
    fn vertical_scroll_offset(&self) -> Twips {
        if self.scroll > 1 {
            let lines = self.layout.lines();

            if let Some(line_data) = lines.get(self.scroll - 1) {
                line_data.offset_y()
            } else {
                Twips::ZERO
            }
        } else {
            Twips::ZERO
        }
    }
}

impl<'gc> EditText<'gc> {
    // This seems to be OS-independent
    const INPUT_NEWLINE: char = '\r';

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
            FormatSpans::from_html(
                &text,
                default_format,
                swf_tag.is_multiline(),
                swf_movie.version(),
            )
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

        let font_type = if swf_tag.use_outlines() {
            FontType::Embedded
        } else {
            FontType::Device
        };

        let layout = html::lower_from_text_spans(
            &text_spans,
            context,
            swf_movie.clone(),
            swf_tag.bounds().width() - Twips::from_pixels(Self::INTERNAL_PADDING * 2.0),
            swf_tag.is_word_wrap(),
            font_type,
        );

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

        // Selections are mandatory in AS3.
        let selection = if swf_movie.is_action_script_3() {
            Some(TextSelection::for_position(text_spans.text().len()))
        } else {
            None
        };

        let et = EditText(GcCell::new(
            context.gc_context,
            EditTextData {
                base,
                text_spans,
                static_data: Gc::new(
                    context.gc_context,
                    EditTextStatic {
                        swf: swf_movie,
                        id: swf_tag.id(),
                        layout: swf_tag.layout().cloned(),
                        initial_text: swf_tag
                            .initial_text()
                            .map(|s| s.decode(encoding).into_owned()),
                    },
                ),
                flags,
                background_color: Color::WHITE,
                border_color: Color::BLACK,
                object: None,
                layout,
                bounds: swf_tag.bounds().clone(),
                autosize,
                requested_width: swf_tag.bounds().width(),
                requested_height: swf_tag.bounds().height(),
                variable: variable.map(|s| s.to_string_lossy(encoding)),
                bound_stage_object: None,
                selection,
                render_settings: Default::default(),
                hscroll: 0.0,
                scroll: 1,
                max_chars: swf_tag.max_length().unwrap_or_default() as i32,
                mouse_wheel_enabled: true,
                is_tlf: false,
                restrict: EditTextRestrict::allow_all(),
            },
        ));

        if swf_tag.is_auto_size() {
            et.relayout(context);
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
            .with_bounds(Rectangle {
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

    /// Create a new, dynamic `EditText` representing an AVM2 TextLine.
    pub fn new_tlf(
        context: &mut UpdateContext<'_, 'gc>,
        swf_movie: Arc<SwfMovie>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let text = Self::new(context, swf_movie, x, y, width, height);
        text.set_is_tlf(context.gc_context, true);
        text.set_selectable(false, context);

        text
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
                write.static_data.swf.version(),
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

    pub fn is_mouse_wheel_enabled(self) -> bool {
        self.0.read().mouse_wheel_enabled
    }

    pub fn set_mouse_wheel_enabled(self, is_enabled: bool, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).mouse_wheel_enabled = is_enabled;
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

    pub fn restrict(self) -> Option<WString> {
        return self.0.read().restrict.value().map(Into::into);
    }

    pub fn set_restrict(self, text: Option<&WStr>, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).restrict = EditTextRestrict::from(text);
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

    pub fn set_has_background(self, gc_context: &Mutation<'gc>, has_background: bool) {
        self.0
            .write(gc_context)
            .flags
            .set(EditTextFlag::HAS_BACKGROUND, has_background);
        self.invalidate_cached_bitmap(gc_context);
    }

    pub fn background_color(self) -> Color {
        self.0.read().background_color
    }

    pub fn set_background_color(self, gc_context: &Mutation<'gc>, background_color: Color) {
        self.0.write(gc_context).background_color = background_color;
        self.invalidate_cached_bitmap(gc_context);
    }

    pub fn has_border(self) -> bool {
        self.0.read().flags.contains(EditTextFlag::BORDER)
    }

    pub fn set_has_border(self, gc_context: &Mutation<'gc>, has_border: bool) {
        self.0
            .write(gc_context)
            .flags
            .set(EditTextFlag::BORDER, has_border);
        self.invalidate_cached_bitmap(gc_context);
    }

    pub fn border_color(self) -> Color {
        self.0.read().border_color
    }

    pub fn set_border_color(self, gc_context: &Mutation<'gc>, border_color: Color) {
        self.0.write(gc_context).border_color = border_color;
        self.invalidate_cached_bitmap(gc_context);
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

    pub fn is_tlf(self) -> bool {
        self.0.read().is_tlf
    }

    pub fn set_is_tlf(self, gc_context: &Mutation<'gc>, is_tlf: bool) {
        self.0.write(gc_context).is_tlf = is_tlf;
    }

    pub fn draw_layout_boxes(self) -> bool {
        self.0
            .read()
            .flags
            .contains(EditTextFlag::DRAW_LAYOUT_BOXES)
    }

    pub fn set_draw_layout_boxes(self, context: &mut UpdateContext<'_, 'gc>, value: bool) {
        self.0
            .write(context.gc())
            .flags
            .set(EditTextFlag::DRAW_LAYOUT_BOXES, value);
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

    /// Internal padding between the bounds of the EditText and the text.
    /// Applies to each side.
    const INTERNAL_PADDING: f64 = 2.0;

    /// Relayout the `EditText`.
    ///
    /// This function operates exclusively with the text-span representation of
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

        // Determine the internal width available for content layout.
        let content_width = if autosize == AutoSizeMode::None || is_word_wrap {
            edit_text.requested_width - padding
        } else {
            edit_text.bounds.width() - padding
        };

        let font_type = if !edit_text.flags.contains(EditTextFlag::USE_OUTLINES) {
            FontType::Device
        } else if edit_text.is_tlf {
            FontType::EmbeddedCFF
        } else {
            FontType::Embedded
        };

        let new_layout = html::lower_from_text_spans(
            &edit_text.text_spans,
            context,
            movie,
            content_width,
            is_word_wrap,
            font_type,
        );

        edit_text.layout = new_layout;
        // reset scroll
        edit_text.hscroll = 0.0;
        edit_text.scroll = 1;

        let layout_exterior_bounds = edit_text.layout.exterior_bounds();

        if autosize != AutoSizeMode::None {
            if !is_word_wrap {
                // The edit text's bounds needs to have the padding baked in.
                let width = layout_exterior_bounds.width() + padding;
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
                let width = edit_text.requested_width;
                edit_text.bounds.set_width(width);
            }
            let height = layout_exterior_bounds.height() + padding;
            edit_text.bounds.set_height(height);
        } else {
            let width = edit_text.requested_width;
            edit_text.bounds.set_width(width);
            let height = edit_text.requested_height;
            edit_text.bounds.set_height(height);
        }
        drop(edit_text);
        self.invalidate_cached_bitmap(context.gc_context);
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, _context: &mut UpdateContext<'_, 'gc>) -> (Twips, Twips) {
        let exterior_bounds = self.0.read().layout.exterior_bounds();
        (exterior_bounds.width(), exterior_bounds.height())
    }

    /// How far the text can be scrolled right, in pixels.
    pub fn maxhscroll(self) -> f64 {
        let edit_text = self.0.read();

        // word-wrapped text can't be scrolled
        if edit_text.flags.contains(EditTextFlag::WORD_WRAP) {
            return 0.0;
        }

        let base = round_down_to_pixel(
            edit_text.layout.exterior_bounds().width() - edit_text.bounds.width(),
        )
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

        let lines = edit_text.layout.lines();

        if lines.is_empty() {
            return 1;
        }

        let target = lines.last().unwrap().extent_y() - edit_text.bounds.height();

        // minimum line n such that n.offset > max.extent - bounds.height()
        let max_line = lines.iter().find(|&l| target < l.offset_y());
        if let Some(line) = max_line {
            line.index() + 1
        } else {
            // I don't know how this could happen, so return the limit
            lines.last().unwrap().index() + 1
        }
    }

    /// The lowest visible line of text
    pub fn bottom_scroll(self) -> usize {
        let edit_text = self.0.read();

        let lines = edit_text.layout.lines();

        if lines.is_empty() {
            return 1;
        }

        let scroll_offset = lines
            .get(edit_text.scroll - 1)
            .map_or(Twips::ZERO, |l| l.offset_y());
        let target = edit_text.bounds.height() + scroll_offset;

        // Line before first line with extent greater than bounds.height() + line "scroll"'s offset
        let too_far = lines.iter().find(|&l| l.extent_y() > target);
        if let Some(line) = too_far {
            line.index()
        } else {
            // all lines are visible
            lines.last().unwrap().index() + 1
        }
    }

    /// Render a layout box, plus its children.
    fn render_layout_box(self, context: &mut RenderContext<'_, 'gc>, lbox: &LayoutBox<'gc>) {
        let origin = lbox.bounds().origin();

        let edit_text = self.0.read();

        // If text's top is under the textbox's bottom, skip drawing.
        // TODO: FP actually skips drawing a line as soon as its bottom is under the textbox;
        //   Current logic is conservative for safety (and even of this I'm not 100% sure).
        // TODO: we should also cull text that's above the textbox
        //   (instead of culling, this can be implemented as having the loop start from `scrollY`th line)
        //   (maybe we could cull-before-render all glyphs, thus removing the need for masking?)
        // TODO: also cull text that's simply out of screen, just like we cull whole DOs in render_self().
        if origin.y() + Twips::from_pixels(Self::INTERNAL_PADDING)
            - edit_text.vertical_scroll_offset()
            > edit_text.bounds.y_max
        {
            return;
        }

        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(origin.x(), origin.y()),
            ..Default::default()
        });

        let visible_selection = if self.has_focus() {
            edit_text.selection
        } else {
            None
        };

        let caret = if let LayoutContent::Text { start, end, .. } = &lbox.content() {
            if let Some(visible_selection) = visible_selection {
                if visible_selection.is_caret()
                    && !edit_text.flags.contains(EditTextFlag::READ_ONLY)
                    && visible_selection.start() >= *start
                    && visible_selection.end() <= *end
                    && !visible_selection.blinks_now()
                {
                    Some(visible_selection.start() - start)
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
            let baseline = font.get_baseline_for_height(params.height());
            let descent = font.get_descent_for_height(params.height());
            let baseline_adjustment = baseline - params.height();
            let caret_height = baseline + descent;
            let mut caret_x = Twips::ZERO;
            font.evaluate(
                text,
                self.text_transform(color, baseline_adjustment),
                params,
                |pos, transform, glyph: &Glyph, advance, x| {
                    if let Some(glyph_shape_handle) = glyph.shape_handle(context.renderer) {
                        // If it's highlighted, override the color.
                        if matches!(visible_selection, Some(visible_selection) if visible_selection.contains(start + pos)) {
                            // Draw black selection rect
                            self.render_selection(context, x, advance, caret_height);

                            // Set text color to white
                            context.transform_stack.push(&Transform {
                                matrix: transform.matrix,
                                color_transform: ColorTransform::IDENTITY,
                            });
                        } else {
                            context.transform_stack.push(transform);
                        }

                        // Render glyph.
                        context
                            .commands
                            .render_shape(glyph_shape_handle, context.transform_stack.transform());
                        context.transform_stack.pop();
                    }

                    // Update caret position
                    if let Some(caret) = caret {
                        if pos == caret {
                            caret_x = x;
                        } else if caret > 0 && pos == caret - 1 {
                            // The caret may be rendered at the end, after all glyphs.
                            caret_x = x + advance;
                        }
                    }
                },
            );

            if caret.is_some() {
                self.render_caret(context, caret_x, caret_height, color);
            }
        }

        if let Some(drawing) = lbox.as_renderable_drawing() {
            drawing.render(context);
        }

        context.transform_stack.pop();
    }

    fn render_selection(
        self,
        context: &mut RenderContext<'_, 'gc>,
        x: Twips,
        width: Twips,
        height: Twips,
    ) {
        let selection_box = context.transform_stack.transform().matrix
            * Matrix::create_box(
                width.to_pixels() as f32,
                height.to_pixels() as f32,
                x,
                Twips::ZERO,
            );
        context.commands.draw_rect(Color::BLACK, selection_box);
    }

    fn render_caret(
        self,
        context: &mut RenderContext<'_, 'gc>,
        x: Twips,
        height: Twips,
        color: Color,
    ) {
        let mut caret = context.transform_stack.transform().matrix
            * Matrix::create_box_with_rotation(
                1.0,
                height.to_pixels() as f32,
                std::f32::consts::FRAC_PI_2,
                x,
                Twips::ZERO,
            );
        let pixel_snapping = EditTextPixelSnapping::new(context.stage.quality());
        pixel_snapping.apply(&mut caret);
        context.commands.draw_line(color, caret);
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

            let mut parent = self.avm1_parent().unwrap();
            while parent.as_avm1_button().is_some() {
                parent = parent.avm1_parent().unwrap();
            }

            activation.run_with_child_frame_for_display_object(
                "[Text Field Binding]",
                parent,
                self.movie().version(),
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
                        self.movie().version(),
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

    pub fn set_selection(self, selection: Option<TextSelection>, gc_context: &Mutation<'gc>) {
        let mut text = self.0.write(gc_context);
        let old_selection = text.selection;
        if let Some(mut selection) = selection {
            selection.clamp(text.text_spans.text().len());
            text.selection = Some(selection);
        } else {
            text.selection = None;
        }

        if old_selection != text.selection {
            drop(text);
            self.invalidate_cached_bitmap(gc_context);
        }
    }

    pub fn reset_selection_blinking(self, gc_context: &Mutation<'gc>) {
        if let Some(selection) = self.0.write(gc_context).selection.as_mut() {
            selection.reset_blinking();
        }
    }

    pub fn spans(&self) -> Ref<FormatSpans> {
        Ref::map(self.0.read(), |r| &r.text_spans)
    }

    pub fn render_settings(self) -> TextRenderSettings {
        self.0.read().render_settings.clone()
    }

    pub fn set_render_settings(self, gc_context: &Mutation<'gc>, settings: TextRenderSettings) {
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
        self.0.write(context.gc()).scroll = clamped;
        self.invalidate_cached_bitmap(context.gc());
    }

    pub fn max_chars(self) -> i32 {
        self.0.read().max_chars
    }

    pub fn set_max_chars(self, value: i32, context: &mut UpdateContext<'_, 'gc>) {
        self.0.write(context.gc_context).max_chars = value;
    }

    pub fn screen_position_to_index(self, position: Point<Twips>) -> Option<usize> {
        let text = self.0.read();
        let mut position = self.global_to_local(position)?;
        position.x += Twips::from_pixels(Self::INTERNAL_PADDING) + Twips::from_pixels(text.hscroll);
        position.y += Twips::from_pixels(Self::INTERNAL_PADDING) + text.vertical_scroll_offset();

        // First determine which *row* of text is the closest match to the Y position...
        let mut closest_row_extent_y = None;
        for layout_box in text.layout.boxes_iter() {
            if layout_box.is_text_box() {
                if let Some(closest_extent_y) = closest_row_extent_y {
                    if layout_box.bounds().extent_y() > closest_extent_y
                        && position.y >= layout_box.bounds().offset_y()
                    {
                        closest_row_extent_y = Some(layout_box.bounds().extent_y());
                    }
                } else {
                    closest_row_extent_y = Some(layout_box.bounds().extent_y());
                }
            }
        }

        // ...then find the box within that row that is the closest match to the X position.
        let mut closest_layout_box: Option<&LayoutBox<'gc>> = None;
        if let Some(closest_extent_y) = closest_row_extent_y {
            for layout_box in text.layout.boxes_iter() {
                if layout_box.is_text_box() {
                    match layout_box.bounds().extent_y().cmp(&closest_extent_y) {
                        Ordering::Less => {}
                        Ordering::Equal => {
                            if position.x >= layout_box.bounds().offset_x()
                                || closest_layout_box.is_none()
                            {
                                closest_layout_box = Some(layout_box);
                            } else {
                                break;
                            }
                        }
                        Ordering::Greater => break,
                    }
                }
            }
        }

        if let Some(layout_box) = closest_layout_box {
            let origin = layout_box.bounds().origin();
            let mut matrix = Matrix::translate(origin.x(), origin.y());
            matrix = matrix.inverse().expect("Invertible layout matrix");
            let local_position = matrix * position;

            if let Some((text, _tf, font, params, color)) =
                layout_box.as_renderable_text(text.text_spans.text())
            {
                let mut result = 0;
                let baseline_adjustment =
                    font.get_baseline_for_height(params.height()) - params.height();
                font.evaluate(
                    text,
                    self.text_transform(color, baseline_adjustment),
                    params,
                    |pos, _transform, _glyph: &Glyph, advance, x| {
                        if local_position.x >= x {
                            if local_position.x > x + (advance / 2) {
                                result = string_utils::next_char_boundary(text, pos);
                            } else {
                                result = pos;
                            }
                        }
                    },
                );
                if let LayoutContent::Text { start, .. } = layout_box.content() {
                    return Some(result + start);
                }
            }
        }

        // Should only be reached if there are no text layout boxes at all.
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

    pub fn is_text_control_applicable(
        self,
        control_code: TextControlCode,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> bool {
        if !self.is_editable() && control_code.is_edit_input() {
            return false;
        }

        let Some(selection) = self.selection() else {
            return false;
        };

        match control_code {
            TextControlCode::SelectLeft
            | TextControlCode::SelectLeftWord
            | TextControlCode::SelectLeftLine
            | TextControlCode::SelectLeftDocument
            | TextControlCode::SelectRight
            | TextControlCode::SelectRightWord
            | TextControlCode::SelectRightLine
            | TextControlCode::SelectRightDocument
            | TextControlCode::SelectAll => self.is_selectable(),
            TextControlCode::Copy | TextControlCode::Cut => {
                !self.is_password() && !selection.is_caret()
            }
            TextControlCode::Paste => context.ui.clipboard_available(),
            _ => true,
        }
    }

    pub fn text_control_input(
        self,
        control_code: TextControlCode,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        if !self.is_text_control_applicable(control_code, context) {
            return;
        }

        let Some(selection) = self.selection() else {
            return;
        };

        let mut changed = false;
        let is_selectable = self.is_selectable();
        match control_code {
            TextControlCode::Enter => {
                self.text_input(Self::INPUT_NEWLINE, context);
            }
            TextControlCode::MoveLeft
            | TextControlCode::MoveLeftWord
            | TextControlCode::MoveLeftLine
            | TextControlCode::MoveLeftDocument => {
                let new_pos = if selection.is_caret() {
                    self.find_new_position(control_code, selection.to)
                } else {
                    selection.start()
                };
                self.set_selection(
                    Some(TextSelection::for_position(new_pos)),
                    context.gc_context,
                );
            }
            TextControlCode::MoveRight
            | TextControlCode::MoveRightWord
            | TextControlCode::MoveRightLine
            | TextControlCode::MoveRightDocument => {
                let new_pos = if selection.is_caret() && selection.to < self.text().len() {
                    self.find_new_position(control_code, selection.to)
                } else {
                    selection.end()
                };
                self.set_selection(
                    Some(TextSelection::for_position(new_pos)),
                    context.gc_context,
                );
            }
            TextControlCode::SelectLeft
            | TextControlCode::SelectLeftWord
            | TextControlCode::SelectLeftLine
            | TextControlCode::SelectLeftDocument => {
                if selection.to > 0 {
                    let new_pos = self.find_new_position(control_code, selection.to);
                    self.set_selection(
                        Some(TextSelection::for_range(selection.from, new_pos)),
                        context.gc_context,
                    );
                }
            }
            TextControlCode::SelectRight
            | TextControlCode::SelectRightWord
            | TextControlCode::SelectRightLine
            | TextControlCode::SelectRightDocument => {
                if selection.to < self.text().len() {
                    let new_pos = self.find_new_position(control_code, selection.to);
                    self.set_selection(
                        Some(TextSelection::for_range(selection.from, new_pos)),
                        context.gc_context,
                    )
                }
            }
            TextControlCode::SelectAll => {
                self.set_selection(
                    Some(TextSelection::for_range(0, self.text().len())),
                    context.gc_context,
                );
            }
            TextControlCode::Copy => {
                let text = &self.text()[selection.start()..selection.end()];
                context.ui.set_clipboard_content(text.to_string());
            }
            TextControlCode::Paste => 'paste: {
                let text = context.ui.clipboard_content();
                if text.is_empty() {
                    // When the clipboard is empty, nothing is pasted
                    // and the already selected text is not removed.
                    // Note that if the clipboard is not empty, but does not have
                    // any allowed characters, the selected text is removed.
                    break 'paste;
                }

                let mut text = self.0.read().restrict.filter_allowed(&text);

                if text.len() > self.available_chars() && self.available_chars() > 0 {
                    text = text[0..self.available_chars()].to_owned();
                }

                if text.len() <= self.available_chars() {
                    self.replace_text(
                        selection.start(),
                        selection.end(),
                        &WString::from_utf8(&text),
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
                let text = &self.text()[selection.start()..selection.end()];
                context.ui.set_clipboard_content(text.to_string());

                self.replace_text(selection.start(), selection.end(), WStr::empty(), context);
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
            TextControlCode::Backspace
            | TextControlCode::BackspaceWord
            | TextControlCode::Delete
            | TextControlCode::DeleteWord
                if !selection.is_caret() =>
            {
                // Backspace or delete with multiple characters selected
                self.replace_text(selection.start(), selection.end(), WStr::empty(), context);
                self.set_selection(
                    Some(TextSelection::for_position(selection.start())),
                    context.gc_context,
                );
                changed = true;
            }
            TextControlCode::Backspace | TextControlCode::BackspaceWord => {
                // Backspace with caret
                if selection.start() > 0 {
                    // Delete previous character(s)
                    let start = self.find_new_position(control_code, selection.start());
                    self.replace_text(start, selection.start(), WStr::empty(), context);
                    self.set_selection(
                        Some(TextSelection::for_position(start)),
                        context.gc_context,
                    );
                    changed = true;
                }
            }
            TextControlCode::Delete | TextControlCode::DeleteWord => {
                // Delete with caret
                if selection.end() < self.text_length() {
                    // Delete next character(s)
                    let end = self.find_new_position(control_code, selection.start());
                    self.replace_text(selection.start(), end, WStr::empty(), context);
                    // No need to change selection, reset it to prevent caret from blinking
                    self.reset_selection_blinking(context.gc_context);
                    changed = true;
                }
            }
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

    /// Find the new position in the text for the given control code.
    ///
    /// * For selection codes it will represent the "to" part of the selection.
    /// * For left/right moves it will represent the final caret position.
    /// * For backspace/delete it will represent the position to which the text should be deleted.
    fn find_new_position(self, control_code: TextControlCode, current_pos: usize) -> usize {
        match control_code {
            TextControlCode::SelectRight | TextControlCode::MoveRight | TextControlCode::Delete => {
                string_utils::next_char_boundary(&self.text(), current_pos)
            }
            TextControlCode::SelectLeft
            | TextControlCode::MoveLeft
            | TextControlCode::Backspace => {
                string_utils::prev_char_boundary(&self.text(), current_pos)
            }
            TextControlCode::SelectRightWord
            | TextControlCode::MoveRightWord
            | TextControlCode::DeleteWord => self.find_next_word_boundary(current_pos),
            TextControlCode::SelectLeftWord
            | TextControlCode::MoveLeftWord
            | TextControlCode::BackspaceWord => self.find_prev_word_boundary(current_pos),
            TextControlCode::SelectRightLine | TextControlCode::MoveRightLine => {
                self.find_next_line_boundary(current_pos)
            }
            TextControlCode::SelectLeftLine | TextControlCode::MoveLeftLine => {
                self.find_prev_line_boundary(current_pos)
            }
            TextControlCode::SelectRightDocument | TextControlCode::MoveRightDocument => {
                self.text().len()
            }
            TextControlCode::SelectLeftDocument | TextControlCode::MoveLeftDocument => 0,
            _ => unreachable!(),
        }
    }

    /// Find the nearest word boundary before `pos`,
    /// which is applicable for selection.
    ///
    /// This algorithm is based on [UAX #29](https://unicode.org/reports/tr29/).
    fn find_prev_word_boundary(self, pos: usize) -> usize {
        let head = &self.text()[..pos];
        let to_utf8 = WStrToUtf8::new(head);
        WordBoundIndices::new(&to_utf8.to_utf8_lossy())
            .rev()
            .find(|(_, span)| !span.trim().is_empty())
            .map(|(position, _)| position)
            .and_then(|utf8_index| to_utf8.utf16_index(utf8_index))
            .unwrap_or(0)
    }

    /// Find the nearest word boundary after `pos`,
    /// which is applicable for selection.
    ///
    /// This algorithm is based on [UAX #29](https://unicode.org/reports/tr29/).
    fn find_next_word_boundary(self, pos: usize) -> usize {
        let tail = &self.text()[pos..];
        let to_utf8 = WStrToUtf8::new(tail);
        WordBoundIndices::new(&to_utf8.to_utf8_lossy())
            .skip_while(|(_, span)| span.trim().is_empty())
            .nth(1)
            .map(|p| p.0)
            .and_then(|utf8_index| to_utf8.utf16_index(utf8_index))
            .map(|utf16_index| pos + utf16_index)
            .unwrap_or_else(|| self.text().len())
    }

    /// Find the nearest line boundary before or at `pos`.
    fn find_prev_line_boundary(self, pos: usize) -> usize {
        // TODO take into account the text layout instead of relying on newlines only
        if pos == 0 {
            return 0;
        }

        let mut line_break_pos = pos;
        while line_break_pos > 0 && !self.is_newline_at(line_break_pos - 1) {
            line_break_pos -= 1;
        }

        line_break_pos
    }

    /// Find the nearest line boundary after or at `pos`.
    fn find_next_line_boundary(self, pos: usize) -> usize {
        // TODO take into account the text layout instead of relying on newlines only
        let len = self.text().len();
        if pos >= len {
            return len;
        }

        let mut line_break_pos = pos;
        while line_break_pos < len && !self.is_newline_at(line_break_pos) {
            line_break_pos += 1;
        }
        line_break_pos
    }

    fn is_newline_at(self, pos: usize) -> bool {
        self.text().get(pos).unwrap_or(0) == '\n' as u16
    }

    pub fn text_input(self, character: char, context: &mut UpdateContext<'_, 'gc>) {
        if self.0.read().flags.contains(EditTextFlag::READ_ONLY)
            || (character.is_control() && character != Self::INPUT_NEWLINE)
            || self.available_chars() == 0
        {
            return;
        }

        if !self.is_multiline() && character == Self::INPUT_NEWLINE {
            return;
        }

        let Some(selection) = self.selection() else {
            return;
        };

        let Some(character) = self.0.read().restrict.to_allowed(character) else {
            return;
        };

        if let Avm2Value::Object(target) = self.object2() {
            let character_string = AvmString::new_utf8(context.gc_context, character.to_string());

            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let text_evt = Avm2EventObject::text_event(
                &mut activation,
                "textInput",
                character_string,
                true,
                true,
            );
            Avm2::dispatch_event(&mut activation.context, text_evt, target);

            if text_evt.as_event().unwrap().is_cancelled() {
                return;
            }
        }

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

        let mut activation = Avm1Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Propagate Text Binding]"),
            self.into(),
        );
        self.propagate_text_binding(&mut activation);
        self.on_changed(&mut activation);
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

    fn on_scroller(&self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Avm1Value::Object(object) = self.object() {
            let _ = object.call_method(
                "broadcastMessage".into(),
                &["onScroller".into(), object.into()],
                activation,
                ExecutionReason::Special,
            );
        }
        //TODO: Implement this for Avm2
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
        self.0.read().layout.lines().len()
    }

    /// Calculate the layout metrics for a given line.
    ///
    /// Returns `None` if the line does not exist or there is not enough data
    /// about the line to calculate metrics with.
    pub fn layout_metrics(self, line: Option<usize>) -> Option<LayoutMetrics> {
        let layout = &self.0.read().layout;
        let line = line.and_then(|line| layout.lines().get(line));

        let (boxes, union_bounds) = if let Some(line) = line {
            (Either::Left(line.boxes_iter()), line.bounds())
        } else {
            (Either::Right(layout.boxes_iter()), layout.bounds())
        };

        let mut first_font = None;
        let mut first_format = None;
        for layout_box in boxes {
            match layout_box.content() {
                LayoutContent::Text {
                    font, text_format, ..
                }
                | LayoutContent::Bullet {
                    font, text_format, ..
                } => {
                    first_font = Some(font);
                    first_format = Some(text_format);
                    break;
                }
                LayoutContent::Drawing { .. } => {}
            }
        }

        let font = first_font?;
        let text_format = first_format?;
        let size = Twips::from_pixels(text_format.size?);
        let ascent = font.get_baseline_for_height(size);
        let descent = font.get_descent_for_height(size);
        let leading = Twips::from_pixels(text_format.leading?);

        Some(LayoutMetrics {
            ascent,
            descent,
            leading,
            width: union_bounds.width(),
            height: union_bounds.height() + descent + leading,
            x: union_bounds.offset_x() + Twips::from_pixels(EditText::INTERNAL_PADDING),
        })
    }

    pub fn line_length(self, line: usize) -> Option<usize> {
        Some(self.0.read().layout.lines().get(line)?.len())
    }

    pub fn line_text(self, line: usize) -> Option<WString> {
        let read = self.0.read();
        let line = read.layout.lines().get(line)?;
        let line_text = read.text_spans.text().slice(line.text_range())?;
        Some(WString::from_wstr(line_text))
    }

    pub fn line_offset(self, line: usize) -> Option<usize> {
        let read = self.0.read();
        let line = read.layout.lines().get(line)?;
        let first_box = line.boxes_iter().next()?;
        Some(first_box.start())
    }

    pub fn line_index_of_char(self, index: usize) -> Option<usize> {
        self.0.read().layout.find_line_index_by_position(index)
    }

    fn execute_avm1_asfunction(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        address: &WStr,
    ) -> Result<(), crate::avm1::Error<'gc>> {
        let Some(parent) = self.avm1_parent() else {
            return Ok(()); // Can't open links for something that isn't visible?
        };

        let mut activation = Avm1Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[EditText URL]"),
            parent,
        );
        // [NA]: Should all `from_nothings` be scoped to root? It definitely should here.
        activation.set_scope_to_display_object(parent);
        let this = parent.object().coerce_to_object(&mut activation);

        if let Some((name, args)) = address.split_once(b',') {
            let name = AvmString::new(activation.context.gc_context, name);
            let args = AvmString::new(activation.context.gc_context, args);
            let function = activation.get_variable(name)?;
            function.call_with_default_this(this, name, &mut activation, &[args.into()])?;
        } else {
            let name = AvmString::new(activation.context.gc_context, address);
            let function = activation.get_variable(name)?;
            function.call_with_default_this(this, name, &mut activation, &[])?;
        }
        Ok(())
    }

    fn open_url(self, context: &mut UpdateContext<'_, 'gc>, url: &WStr, target: &WStr) {
        if let Some(address) = url.strip_prefix(WStr::from_units(b"asfunction:")) {
            if let Err(e) = self.execute_avm1_asfunction(context, address) {
                error!("Couldn't execute URL \"{url:?}\": {e:?}");
            }
        } else if let Some(address) = url.strip_prefix(WStr::from_units(b"event:")) {
            if let Avm2Value::Object(object) = self.object2() {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                let text = AvmString::new(activation.context.gc_context, address);
                let event = Avm2EventObject::text_event(&mut activation, "link", text, true, false);

                Avm2::dispatch_event(&mut activation.context, event, object);
            }
        } else {
            context
                .navigator
                .navigate_to_url(&url.to_utf8_lossy(), &target.to_utf8_lossy(), None);
        }
    }

    fn is_link_at(self, point: Point<Twips>) -> bool {
        let text = self.0.read();
        let Some(mut position) = self.global_to_local(point) else {
            return false;
        };
        position.x += Twips::from_pixels(Self::INTERNAL_PADDING) + Twips::from_pixels(text.hscroll);
        position.y += Twips::from_pixels(Self::INTERNAL_PADDING) + text.vertical_scroll_offset();

        text.layout.boxes_iter().any(|layout| {
            layout.is_link()
                && layout
                    .bounds()
                    .contains(Position::from((position.x, position.y)))
        })
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(GcCell::new(gc_context, self.0.read().clone())).into()
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
        if self.movie().is_action_script_3() && matches!(self.object2(), Avm2Value::Null) {
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

        if !self.movie().is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
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

    fn set_x(&self, gc_context: &Mutation<'gc>, x: Twips) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.x_min;
        edit_text.base.base.set_x(x - offset);
        drop(edit_text);
        self.invalidate_cached_bitmap(gc_context);
    }

    fn y(&self) -> Twips {
        let edit_text = self.0.read();
        let offset = edit_text.bounds.y_min;
        edit_text.base.base.y() + offset
    }

    fn set_y(&self, gc_context: &Mutation<'gc>, y: Twips) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.y_min;
        edit_text.base.base.set_y(y - offset);
        drop(edit_text);
        self.invalidate_cached_bitmap(gc_context);
    }

    fn width(&self) -> f64 {
        let edit_text = self.0.read();
        (edit_text.base.base.transform.matrix * edit_text.bounds.clone())
            .width()
            .to_pixels()
    }

    fn set_width(&self, context: &mut UpdateContext<'_, 'gc>, value: f64) {
        let mut edit_text = self.0.write(context.gc_context);
        edit_text.requested_width = Twips::from_pixels(value);
        edit_text.base.base.set_transformed_by_script(true);
        drop(edit_text);
        self.relayout(context);
    }

    fn height(&self) -> f64 {
        let edit_text = self.0.read();
        (edit_text.base.base.transform.matrix * edit_text.bounds.clone())
            .height()
            .to_pixels()
    }

    fn set_height(&self, context: &mut UpdateContext<'_, 'gc>, value: f64) {
        let mut edit_text = self.0.write(context.gc_context);
        edit_text.requested_height = Twips::from_pixels(value);
        edit_text.base.base.set_transformed_by_script(true);
        drop(edit_text);
        self.relayout(context);
    }

    fn set_matrix(&self, gc_context: &Mutation<'gc>, matrix: Matrix) {
        self.0.write(gc_context).base.base.set_matrix(matrix);
        self.invalidate_cached_bitmap(gc_context);
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        fn is_transform_positive_scale_only(context: &mut RenderContext) -> bool {
            let Matrix { a, b, c, d, .. } = context.transform_stack.transform().matrix;
            b == 0.0 && c == 0.0 && a > 0.0 && d > 0.0
        }

        // EditText is not rendered if device font is used
        // and if it's rotated, sheared, or reflected.
        if self.is_device_font() && !is_transform_positive_scale_only(context) {
            return;
        }

        let edit_text = self.0.read();

        if edit_text
            .flags
            .intersects(EditTextFlag::BORDER | EditTextFlag::HAS_BACKGROUND)
        {
            let background_color = Some(edit_text.background_color)
                .filter(|_| edit_text.flags.contains(EditTextFlag::HAS_BACKGROUND));
            let border_color = Some(edit_text.border_color)
                .filter(|_| edit_text.flags.contains(EditTextFlag::BORDER));

            if self.is_device_font() {
                self.draw_device_text_box(
                    context,
                    edit_text.bounds.clone(),
                    background_color,
                    border_color,
                );
            } else {
                self.draw_text_box(
                    context,
                    edit_text.bounds.clone(),
                    background_color,
                    border_color,
                );
            }
        }

        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(edit_text.bounds.x_min, edit_text.bounds.y_min),
            ..Default::default()
        });

        context.commands.push_mask();
        let mask = Matrix::create_box(
            edit_text.bounds.width().to_pixels() as f32,
            edit_text.bounds.height().to_pixels() as f32,
            Twips::ZERO,
            Twips::ZERO,
        );
        context.commands.draw_rect(
            Color::WHITE,
            context.transform_stack.transform().matrix * mask,
        );
        context.commands.activate_mask();

        let scroll_offset = edit_text.vertical_scroll_offset();
        // TODO: Where does this come from? How is this different than INTERNAL_PADDING? Does this apply to y as well?
        // If this is actually right, offset the border in `redraw_border` instead of doing an extra push.
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(
                Twips::from_pixels(Self::INTERNAL_PADDING) - Twips::from_pixels(edit_text.hscroll),
                Twips::from_pixels(Self::INTERNAL_PADDING) - scroll_offset,
            ),
            ..Default::default()
        });

        if edit_text.layout.boxes_iter().next().is_none()
            && !edit_text.flags.contains(EditTextFlag::READ_ONLY)
        {
            // TODO should not be possible
            let visible_selection = if self.has_focus() {
                edit_text.selection
            } else {
                None
            };
            if let Some(visible_selection) = visible_selection {
                if visible_selection.is_caret()
                    && visible_selection.start() == 0
                    && !visible_selection.blinks_now()
                {
                    let format = edit_text.text_spans.default_format();
                    let caret_height = format.size.map(Twips::from_pixels).unwrap_or_default();
                    self.render_caret(context, Twips::ZERO, caret_height, Color::BLACK);
                }
            }
        } else {
            let draw_boxes = edit_text.flags.contains(EditTextFlag::DRAW_LAYOUT_BOXES);
            if draw_boxes {
                context.draw_rect_outline(
                    Color::GREEN,
                    edit_text.layout.exterior_bounds().into(),
                    Twips::ONE,
                );
            }

            for layout_box in edit_text.layout.boxes_iter() {
                if draw_boxes {
                    context.draw_rect_outline(Color::RED, layout_box.bounds().into(), Twips::ONE);
                }
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
        self.drop_focus(context);

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
}

impl<'gc> EditText<'gc> {
    /// Draw the box (border + background) for EditText with device fonts.
    ///
    /// Notes on FP's behavior:
    ///  * the box is never drawn when there's any rotation, shear, or reflection,
    ///  * the box is always aliased and lines lie on whole pixels regardless of quality,
    ///  * line width of the border is always 1px regardless of zoom and transform,
    ///  * the bottom-right corner of the border is missing.
    ///
    /// Notes on the current implementation:
    ///  * the border is drawn using four separately drawn lines,
    ///  * the lines are always snapped to whole pixels (which is easy as
    ///    the possible transforms are highly limited),
    ///  * the current implementation should be pixel-perfect (compared to FP).
    pub fn draw_device_text_box(
        &self,
        context: &mut RenderContext<'_, 'gc>,
        bounds: Rectangle<Twips>,
        background_color: Option<Color>,
        border_color: Option<Color>,
    ) {
        let transform = context.transform_stack.transform();
        let bounds = transform.matrix * bounds;

        let width_twips = bounds.width().round_to_pixel_ties_even();
        let height_twips = bounds.height().round_to_pixel_ties_even();
        let bounds = Rectangle {
            x_min: bounds.x_min.round_to_pixel_ties_even(),
            x_max: bounds.x_min.round_to_pixel_ties_even() + width_twips,
            y_min: bounds.y_min.round_to_pixel_ties_even(),
            y_max: bounds.y_min.round_to_pixel_ties_even() + height_twips,
        };

        let width = width_twips.to_pixels() as f32;
        let height = height_twips.to_pixels() as f32;
        if let Some(background_color) = background_color {
            let background_color = &transform.color_transform * background_color;
            context.commands.draw_rect(
                background_color,
                Matrix::create_box(width, height, bounds.x_min, bounds.y_min),
            );
        }

        if let Some(border_color) = border_color {
            let border_color = &transform.color_transform * border_color;
            // Top
            context.commands.draw_line(
                border_color,
                Matrix::create_box(width, 1.0, bounds.x_min - Twips::HALF, bounds.y_min),
            );
            // Bottom
            context.commands.draw_line(
                border_color,
                Matrix::create_box(width, 1.0, bounds.x_min - Twips::HALF, bounds.y_max),
            );
            // Left
            context.commands.draw_line(
                border_color,
                Matrix::create_box_with_rotation(
                    1.0,
                    height,
                    std::f32::consts::FRAC_PI_2,
                    bounds.x_min,
                    bounds.y_min - Twips::HALF,
                ),
            );
            // Right
            context.commands.draw_line(
                border_color,
                Matrix::create_box_with_rotation(
                    1.0,
                    height,
                    std::f32::consts::FRAC_PI_2,
                    bounds.x_max,
                    bounds.y_min - Twips::HALF,
                ),
            );
        }
    }

    /// Draw the box (border + background) for EditText with embedded fonts.
    ///
    /// Notes on FP's behavior:
    ///  * the box is always drawn (in contrast to device fonts) and may be transformed,
    ///  * the box is anti aliased according to the quality, but
    ///    is snapped to whole pixels in order not to look blurry,
    ///  * however, on some qualities (e.g. medium) the border is sometimes drawn between pixels,
    ///  * similarly for small box sizes, the border will be sometimes drawn between pixels,
    ///  * line width of the border is always 1px regardless of zoom and transform,
    ///  * the bottom-right corner of the border is NOT missing (usually), :)
    ///  * however, sometimes the bottom-right corner will
    ///    stick out a bit down (gee, can you even draw a rectangle Adobe?),
    ///  * pixel snapping for width sometimes depends on x,
    ///    but pixel snapping for height never depends on y (for high quality).
    ///
    /// Notes on the current implementation:
    ///  * the box is rendered using a line rect,
    ///    which is snapped to pixels using [`EditTextPixelSnapping`],
    ///  * the pixel-perfect position is really hard to achieve, currently it's best-effort only.
    pub fn draw_text_box(
        &self,
        context: &mut RenderContext<'_, 'gc>,
        bounds: Rectangle<Twips>,
        background_color: Option<Color>,
        border_color: Option<Color>,
    ) {
        let quality = context.stage.quality();
        let pixel_snapping = &EditTextPixelSnapping::new(quality);

        let transform = context.transform_stack.transform();

        let width = bounds.width().to_pixels() as f32;
        let height = bounds.height().to_pixels() as f32;

        let mut text_box =
            transform.matrix * Matrix::create_box(width, height, bounds.x_min, bounds.y_min);
        pixel_snapping.apply(&mut text_box);

        if let Some(background_color) = background_color {
            let background_color = &transform.color_transform * background_color;
            context.commands.draw_rect(background_color, text_box);
        }

        if let Some(border_color) = border_color {
            let border_color = &transform.color_transform * border_color;
            context.commands.draw_line_rect(border_color, text_box);
        }
    }
}

impl<'gc> TInteractiveObject<'gc> for EditText<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn raw_interactive_mut(&self, mc: &Mutation<'gc>) -> RefMut<InteractiveObjectBase<'gc>> {
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
        match event {
            ClipEvent::Press | ClipEvent::MouseWheel { .. } | ClipEvent::MouseMove => {
                ClipEventResult::Handled
            }
            _ => ClipEventResult::NotHandled,
        }
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if let ClipEvent::MouseWheel { delta } = event {
            if self.is_mouse_wheel_enabled() {
                let new_scroll = self.scroll() as f64 - delta.lines();
                self.set_scroll(new_scroll, context);

                let mut activation = Avm1Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[On Scroller]"),
                    self.into(),
                );
                self.on_scroller(&mut activation);
            }
            return ClipEventResult::Handled;
        }

        if let ClipEvent::Press = event {
            if self.is_editable() || self.is_selectable() {
                let tracker = context.focus_tracker;
                tracker.set(Some(self.into()), context);
            }

            // We can't hold self as any link may end up modifying this object, so pull the info out
            let mut link_to_open = None;

            if let Some(position) = self.screen_position_to_index(*context.mouse_position) {
                self.set_selection(Some(TextSelection::for_position(position)), context.gc());

                if let Some((span_index, _)) =
                    self.0.read().text_spans.resolve_position_as_span(position)
                {
                    link_to_open = self
                        .0
                        .read()
                        .text_spans
                        .span(span_index)
                        .map(|s| (s.url.clone(), s.target.clone()));
                }
            } else {
                self.set_selection(
                    Some(TextSelection::for_position(self.text_length())),
                    context.gc(),
                );
            }

            if let Some((url, target)) = link_to_open {
                if !url.is_empty() {
                    // TODO: This fires on mouse DOWN but it should be mouse UP...
                    // but only if it went down in the same span.
                    // Needs more advanced focus handling than we have at time of writing this comment.
                    self.open_url(context, &url, &target);
                }
            }

            return ClipEventResult::Handled;
        }

        if let ClipEvent::MouseMove = event {
            // If a mouse has moved and this EditTest is pressed, we need to update the selection.
            if InteractiveObject::option_ptr_eq(context.mouse_data.pressed, self.as_interactive()) {
                if let Some(mut selection) = self.selection() {
                    if let Some(position) = self.screen_position_to_index(*context.mouse_position) {
                        selection.to = position;
                        self.set_selection(Some(selection), context.gc());
                    }
                }
            }
        }

        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // Don't do anything if run in an AVM2 context.
        if self.as_displayobject().movie().is_action_script_3() {
            return None;
        }

        // The text is hovered if the mouse is over any child nodes.
        if self.visible()
            && self.mouse_enabled()
            && (self.is_selectable() || self.is_link_at(point))
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
        // Don't do anything if run in an AVM1 context.
        if !self.as_displayobject().movie().is_action_script_3() {
            return Avm2MousePick::Miss;
        }

        // The text is hovered if the mouse is over any child nodes.
        if self.visible() && self.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
            // Note - for mouse-enabled selectable text, we consider this to be a hit (which
            // will cause us to show the proper cursor on mouse over).
            // However, in `Interactive::event_dispatch_to_avm2`, we will prevent mouse events
            // from being fired at all if the text is selectable and 'was_static()'.
            if self.mouse_enabled()
                && (self.is_selectable() || self.is_link_at(point) || !self.was_static())
            {
                Avm2MousePick::Hit((*self).into())
            } else {
                Avm2MousePick::PropagateToParent
            }
        } else {
            Avm2MousePick::Miss
        }
    }

    fn mouse_cursor(self, context: &mut UpdateContext<'_, 'gc>) -> MouseCursor {
        if self.is_link_at(*context.mouse_position) {
            MouseCursor::Hand
        } else if self.is_selectable() {
            MouseCursor::IBeam
        } else {
            MouseCursor::Arrow
        }
    }

    fn on_focus_changed(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        focused: bool,
        _other: Option<InteractiveObject<'gc>>,
    ) {
        let is_avm1 = !self.movie().is_action_script_3();
        if !focused && is_avm1 {
            self.set_selection(None, context.gc_context);
        }
    }

    fn is_highlightable(&self, _context: &mut UpdateContext<'_, 'gc>) -> bool {
        // TextField is incapable of rendering a highlight.
        false
    }

    fn is_tabbable(&self, context: &mut UpdateContext<'_, 'gc>) -> bool {
        if !self.is_editable() {
            // Non-editable text fields are never tabbable.
            return false;
        }
        self.tab_enabled(context)
    }

    fn tab_enabled_default(&self, _context: &mut UpdateContext<'_, 'gc>) -> bool {
        self.is_editable()
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    struct EditTextFlag: u16 {
        const FIRING_VARIABLE_BINDING = 1 << 0;
        const HAS_BACKGROUND = 1 << 1;
        const DRAW_LAYOUT_BOXES = 1 << 2;

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
    layout: Option<swf::TextLayout>,
    initial_text: Option<WString>,
}

#[derive(Copy, Clone, Debug)]
pub struct TextSelection {
    from: usize,
    to: usize,

    /// The time the caret should begin blinking
    blink_epoch: DateTime<Utc>,
}

impl PartialEq for TextSelection {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for TextSelection {}

impl TextSelection {
    const BLINK_CYCLE_DURATION_MS: u32 = 1000;

    pub fn for_position(position: usize) -> Self {
        Self {
            from: position,
            to: position,
            blink_epoch: Utc::now(),
        }
    }

    pub fn for_range(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            blink_epoch: Utc::now(),
        }
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

    pub fn reset_blinking(&mut self) {
        self.blink_epoch = Utc::now();
    }

    /// Returns true if the caret should not be visible now due to blinking.
    pub fn blinks_now(&self) -> bool {
        let millis = (Utc::now() - self.blink_epoch).num_milliseconds() as u32;
        2 * (millis % Self::BLINK_CYCLE_DURATION_MS) >= Self::BLINK_CYCLE_DURATION_MS
    }
}

#[derive(Clone, Debug)]
struct EditTextRestrict {
    /// Original string value.
    value: Option<WString>,

    /// List of intervals (inclusive, inclusive) with allowed characters.
    allowed: Vec<(char, char)>,

    /// List of intervals (inclusive, inclusive) with disallowed characters.
    disallowed: Vec<(char, char)>,
}

enum EditTextRestrictToken {
    Char(char),
    Range,
    Caret,
}

impl EditTextRestrict {
    const INTERVAL_ALL: (char, char) = ('\0', char::MAX);

    pub fn allow_all() -> Self {
        Self {
            value: None,
            allowed: vec![Self::INTERVAL_ALL],
            disallowed: vec![],
        }
    }

    pub fn allow_none() -> Self {
        Self {
            value: Some(WString::new()),
            allowed: vec![],
            disallowed: vec![],
        }
    }

    pub fn from(value: Option<&WStr>) -> Self {
        match value {
            None => Self::allow_all(),
            Some(string) => Self::from_string(string),
        }
    }

    pub fn from_string(string: &WStr) -> Self {
        if string.is_empty() {
            return Self::allow_none();
        }

        let mut tokens = Self::tokenize_restrict(string);
        let mut allowed: Vec<(char, char)> = vec![];
        let mut disallowed: Vec<(char, char)> = vec![];

        Self::parse_restrict(&mut tokens, &mut allowed, &mut disallowed);

        Self {
            value: Some(string.into()),
            allowed,
            disallowed,
        }
    }

    fn tokenize_restrict(string: &WStr) -> VecDeque<EditTextRestrictToken> {
        let mut characters: VecDeque<char> = string
            .chars()
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<VecDeque<char>>();
        let mut tokens: VecDeque<EditTextRestrictToken> = VecDeque::with_capacity(characters.len());

        while !characters.is_empty() {
            match characters.pop_front().unwrap() {
                // Handle escapes: \\, \-, \^.
                // In fact, other escapes also work, so that \a is equivalent to a, not to \\a.
                '\\' => {
                    if let Some(escaped) = characters.pop_front() {
                        tokens.push_back(EditTextRestrictToken::Char(escaped));
                    } else {
                        // Ignore truncated escapes (when the string ends with \).
                    }
                }
                '^' => {
                    tokens.push_back(EditTextRestrictToken::Caret);
                }
                '-' => {
                    tokens.push_back(EditTextRestrictToken::Range);
                }
                c => {
                    tokens.push_back(EditTextRestrictToken::Char(c));
                }
            }
        }

        tokens
    }

    fn parse_restrict(
        tokens: &mut VecDeque<EditTextRestrictToken>,
        allowed: &mut Vec<(char, char)>,
        disallowed: &mut Vec<(char, char)>,
    ) {
        let mut current_intervals: Vec<(char, char)> = vec![];
        let mut last_char: Option<char> = None;
        let mut now_allowing = true;
        while !tokens.is_empty() {
            last_char = match tokens.pop_front().unwrap() {
                EditTextRestrictToken::Char(c) => {
                    current_intervals.push((c, c));
                    Some(c)
                }
                EditTextRestrictToken::Caret => {
                    if now_allowing {
                        if current_intervals.is_empty() && allowed.is_empty() {
                            // If restrict starts with ^, we are assuming that
                            // all characters are allowed and disallowing from that.
                            allowed.append(&mut vec![Self::INTERVAL_ALL]);
                        } else {
                            allowed.append(&mut current_intervals);
                        }
                    } else {
                        disallowed.append(&mut current_intervals);
                    }

                    // Caret according to the documentation indicates
                    // that we are now disallowing characters.
                    // In reality it just switches allowing/disallowing.
                    now_allowing = !now_allowing;
                    None
                }
                EditTextRestrictToken::Range => {
                    let range_start = if let Some(last_char) = last_char {
                        current_intervals.pop();
                        last_char
                    } else {
                        // When the range is truncated from the left side (-z),
                        // it is equivalent to \0-z.
                        '\0'
                    };
                    let range_end;
                    if let Some(EditTextRestrictToken::Char(c)) = tokens.front() {
                        range_end = *c;
                        tokens.pop_front();
                    } else {
                        // When the range is truncated from the right side (a-),
                        // it is equivalent to the first character (a).
                        range_end = range_start;
                    }
                    // If the range a-z is inverted (z-a), it is equivalent to
                    // the first character only (z).
                    current_intervals.push((range_start, range_end.max(range_start)));
                    None
                }
            }
        }

        if now_allowing {
            allowed.append(&mut current_intervals);
        } else {
            disallowed.append(&mut current_intervals);
        }
    }

    pub fn value(&self) -> Option<&WStr> {
        self.value.as_deref()
    }

    pub fn is_allowed(&self, character: char) -> bool {
        self.intervals_contain(character, &self.allowed)
            && !self.intervals_contain(character, &self.disallowed)
    }

    fn intervals_contain(&self, character: char, intervals: &Vec<(char, char)>) -> bool {
        for interval in intervals {
            if self.interval_contains(character, interval) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn interval_contains(&self, character: char, interval: &(char, char)) -> bool {
        character >= interval.0 && character <= interval.1
    }

    pub fn to_allowed(&self, character: char) -> Option<char> {
        if self.is_allowed(character) {
            Some(character)
        } else if self.is_allowed(character.to_ascii_uppercase()) {
            Some(character.to_ascii_uppercase())
        } else if self.is_allowed(character.to_ascii_lowercase()) {
            Some(character.to_ascii_lowercase())
        } else {
            None
        }
    }

    pub fn filter_allowed(&self, text: &str) -> String {
        let mut filtered = String::with_capacity(text.len());
        for c in text.chars() {
            if let Some(c) = self.to_allowed(c) {
                filtered.push(c);
            }
        }
        filtered
    }
}

#[derive(Debug, Clone)]
struct EditTextPixelSnapping {
    quality: StageQuality,
}

impl EditTextPixelSnapping {
    pub fn new(quality: StageQuality) -> Self {
        Self { quality }
    }

    pub fn apply(&self, matrix: &mut Matrix) {
        match self.quality {
            StageQuality::Low => {
                // We are snapping x and y in order to match the expected positions.
                // However, we do not need to snap scale, because
                // at low quality antialiasing is disabled anyway,
                // and the aliased border is pretty close to the expected position.
                matrix.tx = matrix.tx.round_to_pixel_ties_even();
                matrix.ty = matrix.ty.round_to_pixel_ties_even();
            }
            _ => {
                // For higher qualities, we need to snap x, y, and scales not only to match
                // FP's positioning, but also for the border not to look blurry.
                // The snapping here is fine-tuned for high quality (the default).
                // It is not perfect (FP's logic is very complicated), but it's
                // accurate for whole-pixel positions and relatively close for subpixel positions.
                matrix.tx = (matrix.tx + Twips::new(2)).trunc_to_pixel();
                matrix.ty = (matrix.ty + Twips::new(2)).trunc_to_pixel();
                let x_snap = matrix.c.abs() < 0.001 || matrix.d.abs() < 0.001;
                let y_snap = matrix.a.abs() < 0.001 || matrix.b.abs() < 0.001;
                if x_snap {
                    matrix.a = (matrix.a - 0.35).round_ties_even();
                    matrix.b = (matrix.b - 0.35).round_ties_even();
                }
                if y_snap {
                    matrix.c = (matrix.c - 0.35).round_ties_even();
                    matrix.d = (matrix.d - 0.35).round_ties_even();
                }
            }
        }
    }
}
