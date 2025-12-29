//! `EditText` display object and support code.

use crate::avm1::{
    Activation as Avm1Activation, ActivationIdentifier, Avm1, ExecutionReason,
    NativeObject as Avm1NativeObject, Object as Avm1Object, Value as Avm1Value,
};
use crate::avm2::object::{
    ClassObject as Avm2ClassObject, EventObject as Avm2EventObject, StageObject as Avm2StageObject,
    StyleSheetObject as Avm2StyleSheetObject,
};
use crate::avm2::{Activation as Avm2Activation, Avm2};
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{Avm1TextFieldBinding, DisplayObjectBase};
use crate::events::{
    ClipEvent, ClipEventResult, ImeCursorArea, ImeEvent, ImeNotification, ImePurpose,
    PlayerNotification, TextControlCode,
};
use crate::font::{FontLike, FontType, TextRenderSettings};
use crate::html;
use crate::html::StyleSheet;
use crate::html::{
    FormatSpans, Layout, LayoutBox, LayoutContent, LayoutLine, LayoutMetrics, Position, TextFormat,
};
use crate::prelude::*;
use crate::string::{AvmString, SwfStrExt as _, WStr, WString, utils as string_utils};
use crate::tag_utils::SwfMovie;
use crate::vminterface::{AvmObject, Instantiator};
use chrono::DateTime;
use chrono::Utc;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use ruffle_render::commands::Command as RenderCommand;
use ruffle_render::commands::CommandHandler;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use ruffle_wstr::WStrToUtf8;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::sync::Arc;
use swf::ColorTransform;
use unicode_segmentation::UnicodeSegmentation;

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
pub struct EditText<'gc>(Gc<'gc, EditTextData<'gc>>);

impl fmt::Debug for EditText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EditText")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct EditTextData<'gc> {
    /// DisplayObject and InteractiveObject common properties.
    base: InteractiveObjectBase<'gc>,

    /// Data shared among all instances of this `EditText`.
    shared: Gc<'gc, EditTextShared>,

    /// The AVM1 object handle
    object: Lock<Option<AvmObject<'gc>>>,

    /// The variable path that this text field is bound to (AVM1 only).
    variable: Lock<Option<AvmString<'gc>>>,

    /// The (AVM1) display object that the variable binding is bound to.
    bound_display_object: Lock<Option<DisplayObject<'gc>>>,

    /// Other AVM1 text fields bound to *this* text field.
    avm1_text_field_bindings: RefLock<Vec<Avm1TextFieldBinding<'gc>>>,

    /// The AVM2 class of this button. If None, it is flash.text.TextField.
    class: Lock<Option<Avm2ClassObject<'gc>>>,

    /// The underlying text format spans of the `EditText`.
    ///
    /// This is generated from HTML (with optional CSS) or set directly, and
    /// can be directly manipulated by ActionScript. It can also be raised to
    /// an equivalent HTML representation, as long as no stylesheet is present.
    ///
    /// It is lowered further into layout boxes, which are used for actual
    /// rendering.
    text_spans: RefCell<FormatSpans>,

    /// The calculated layout.
    layout: RefLock<Layout<'gc>>,

    /// Style sheet used when parsing HTML.
    style_sheet: Lock<EditTextStyleSheet<'gc>>,

    /// Restrict what characters the user may input.
    restrict: RefCell<EditTextRestrict>,

    /// Information related to the last click event inside this text field.
    last_click: Cell<Option<ClickEventData>>,

    /// Original HTML text before parsing.
    ///
    /// It is used only when a style sheet is available
    /// in order to preserve styles.
    original_html_text: RefCell<Option<WString>>,

    ime_data: RefCell<Option<ImeData>>,

    /// The color of the background fill. Only applied when has_border and has_background.
    background_color: Cell<Color>,

    /// The color of the border.
    border_color: Cell<Color>,

    /// The selected portion of the text, or None if the text is not selected.
    /// Note: Selections work differently in AVM1, AVM2, and Ruffle.
    ///
    /// In AVM1, there is one global optional selection. If present, it applies to whatever text field is focused.
    /// In AVM2, every text field has its own mandatory selection.
    /// In Ruffle, every text field has its own optional selection. This hybrid approach means manually maintaining
    /// the invariants that selection is always None for an unfocused AVM1 field, and never None for an AVM2 field.
    selection: Cell<Option<TextSelection>>,

    /// The current intrinsic bounds of the text field.
    bounds: Cell<Rectangle<Twips>>,

    /// Which rendering engine this text field will use.
    render_settings: Cell<TextRenderSettings>,

    /// How many pixels right the text is offset by. 0-based index.
    hscroll: Cell<f64>,

    /// How many lines down the text is offset by. 1-based index.
    scroll: Cell<usize>,

    /// The limit of characters that can be manually input by the user.
    /// Doesn't affect script-triggered modifications.
    max_chars: Cell<i32>,

    /// Lazily calculated autosize bounds.
    ///
    /// When `None`, no new bounds should be applied.
    /// When `Some`, new bounds resulting from autosize are
    /// waiting to be applied, see [`EditText::apply_autosize_bounds`].
    autosize_lazy_bounds: Cell<Option<Rectangle<Twips>>>,

    /// Whether the width of the field should change in response to text
    /// changes, and in what direction the added or removed width should
    /// apply.
    autosize: Cell<AutoSizeMode>,

    /// Indicates if the text is scrollable using the mouse wheel.
    mouse_wheel_enabled: Cell<bool>,

    /// Flags indicating the text field's settings.
    flags: Cell<EditTextFlag>,

    /// Flags specifying how layout debug boxes should be drawn.
    layout_debug_boxes_flags: Cell<LayoutDebugBoxesFlag>,

    /// Whether this EditText represents an AVM2 TextLine.
    ///
    /// FTE (Flash Text Engine) is a low-level API for sophisticated text control.
    ///
    /// See <https://docs.ruffle.rs/en_US/FlashPlatform/reference/actionscript/3/flash/text/engine/TextLine.html>
    /// See <https://docs.ruffle.rs/en_US/FlashPlatform/reference/actionscript/3/flash/text/engine/package-detail.html>
    /// See <https://docs.ruffle.rs/en_US/as3/dev/WS9dd7ed846a005b294b857bfa122bd808ea6-8000.html>
    is_fte: Cell<bool>,
}

impl EditTextData<'_> {
    fn vertical_scroll_offset(&self) -> Twips {
        if self.scroll.get() > 1 {
            let layout = self.layout.borrow();
            let lines = layout.lines();

            if let Some(line_data) = lines.get(self.scroll.get() - 1) {
                line_data.offset_y()
            } else {
                Twips::ZERO
            }
        } else {
            Twips::ZERO
        }
    }

    fn font_type(&self) -> FontType {
        if !self.flags.get().contains(EditTextFlag::USE_OUTLINES) {
            FontType::Device
        } else if self.is_fte.get() {
            FontType::EmbeddedCFF
        } else {
            FontType::Embedded
        }
    }

    fn parse_html(&self, text: &WStr) {
        let default_format = self.text_spans.borrow().default_format().clone();
        self.text_spans.replace(FormatSpans::from_html(
            text,
            default_format,
            self.style_sheet.get().style_sheet(),
            self.flags.get().contains(EditTextFlag::MULTILINE),
            self.flags.get().contains(EditTextFlag::CONDENSE_WHITE),
            self.shared.swf.version(),
        ));
        self.original_html_text
            .replace(if self.style_sheet.get().is_some() {
                Some(text.to_owned())
            } else {
                None
            });
    }
}

impl<'gc> EditText<'gc> {
    const ANY_NEWLINE: [char; 2] = ['\n', '\r'];

    // This seems to be OS-independent
    const INPUT_NEWLINE: char = '\r';

    /// Gutter is the constant internal padding of a text field.
    /// It applies to each side and cannot be changed.
    ///
    /// See <https://open-flash.github.io/mirrors/as2-language-reference/TextFormat.html#getTextExtent()>.
    /// See <https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextLineMetrics.html>.
    const GUTTER: Twips = Twips::new(40);

    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'gc>,
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
                None,
                swf_tag.is_multiline(),
                false,
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

        let is_word_wrap = swf_tag.is_word_wrap();
        let content_width = if autosize == AutoSizeMode::None || is_word_wrap {
            Some(swf_tag.bounds().width() - Self::GUTTER * 2)
        } else {
            None
        };

        let layout = html::lower_from_text_spans(
            &text_spans,
            context,
            swf_movie.clone(),
            content_width,
            !swf_tag.is_read_only(),
            is_word_wrap,
            font_type,
        );

        let variable = if !swf_tag.variable_name().is_empty() {
            Some(swf_tag.variable_name().decode(encoding))
        } else {
            None
        };
        let variable = variable.map(|s| context.strings.intern_wstr(s).into());

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

        let et = EditText(Gc::new(
            context.gc(),
            EditTextData {
                base: Default::default(),
                text_spans: RefCell::new(text_spans),
                shared: Gc::new(
                    context.gc(),
                    EditTextShared {
                        swf: swf_movie,
                        id: swf_tag.id(),
                        initial_text: swf_tag
                            .initial_text()
                            .map(|s| s.decode(encoding).into_owned()),
                    },
                ),
                flags: Cell::new(flags),
                background_color: Cell::new(Color::WHITE),
                border_color: Cell::new(Color::BLACK),
                object: Lock::new(None),
                layout: RefLock::new(layout),
                bounds: Cell::new(*swf_tag.bounds()),
                autosize_lazy_bounds: Cell::new(None),
                autosize: Cell::new(autosize),
                variable: Lock::new(variable),
                bound_display_object: Lock::new(None),
                class: Lock::new(None),
                selection: Cell::new(selection),
                render_settings: Default::default(),
                hscroll: Cell::new(0.0),
                scroll: Cell::new(1),
                max_chars: Cell::new(swf_tag.max_length().unwrap_or_default() as i32),
                mouse_wheel_enabled: Cell::new(true),
                is_fte: Cell::new(false),
                restrict: RefCell::new(EditTextRestrict::allow_all()),
                last_click: Cell::new(None),
                layout_debug_boxes_flags: Cell::new(LayoutDebugBoxesFlag::empty()),
                style_sheet: Lock::new(EditTextStyleSheet::None),
                original_html_text: RefCell::new(None),
                ime_data: RefCell::new(None),
                avm1_text_field_bindings: RefLock::new(Vec::new()),
            },
        ));

        if swf_tag.is_auto_size() {
            et.relayout(context);
        }

        et
    }

    /// Create a new, dynamic `EditText`.
    pub fn new(
        context: &mut UpdateContext<'gc>,
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
            let base = text_field.base();
            let mut matrix = base.matrix();
            matrix.tx = Twips::from_pixels(x);
            matrix.ty = Twips::from_pixels(y);
            base.set_matrix(matrix);
        }

        text_field
    }

    /// Create a new, dynamic `EditText` representing an AVM2 TextLine.
    pub fn new_fte(
        context: &mut UpdateContext<'gc>,
        swf_movie: Arc<SwfMovie>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let text = Self::new(context, swf_movie, x, y, width, height);
        text.set_is_fte(true);
        text.set_selectable(false);

        text
    }

    fn contains_flag(self, flag: EditTextFlag) -> bool {
        self.0.flags.get().contains(flag)
    }

    fn set_flag(self, flag: EditTextFlag, value: bool) {
        let mut flags = self.0.flags.get();
        flags.set(flag, value);
        self.0.flags.set(flags);
    }

    fn bounds_x_offset(self) -> Twips {
        let scale_x = self.base().scale_x().unit();
        let offset = self.0.bounds.get().x_min.to_pixels();
        Twips::from_pixels(scale_x * offset)
    }

    fn bounds_y_offset(self) -> Twips {
        let scale_y = self.base().scale_y().unit();
        let offset = self.0.bounds.get().y_min.to_pixels();
        Twips::from_pixels(scale_y * offset)
    }

    pub fn text(self) -> WString {
        self.0.text_spans.borrow().text().into()
    }

    pub fn set_text(self, text: &WStr, context: &mut UpdateContext<'gc>) {
        if self.text() == text {
            // Note: this check not only prevents text relayout,
            // but it also has observable effects, because text
            // format is not being reset to the default format.
            return;
        }

        if self.0.style_sheet.get().is_some() {
            // When CSS is set, text will always be treated as HTML.
            self.0.parse_html(text);
        } else {
            let default_format = self.0.text_spans.borrow().default_format().clone();
            self.0
                .text_spans
                .replace(FormatSpans::from_text(text.into(), default_format));
        }

        self.relayout(context);
    }

    pub fn html_text(self) -> WString {
        if self.is_effectively_html() {
            if let Some(html) = self.0.original_html_text.borrow().clone() {
                return html;
            }

            self.0.text_spans.borrow().to_html()
        } else {
            // Non-HTML text fields always return plain text.
            self.text()
        }
    }

    pub fn set_html_text(self, text: &WStr, context: &mut UpdateContext<'gc>) {
        if self.html_text() == text {
            // Note: this check not only prevents text relayout,
            // but it also has observable effects, because not
            // every set of spans is representable as HTML.
            //
            // For instance, a paragraph may not end with a newline,
            // but its HTML representation will always infer one.
            return;
        }

        if self.is_effectively_html() {
            self.0.parse_html(text);
            self.relayout(context);
        } else {
            self.set_text(text, context);
        }
    }

    pub fn text_length(self) -> usize {
        self.0.text_spans.borrow().text().len()
    }

    pub fn new_text_format(self) -> TextFormat {
        self.0.text_spans.borrow().default_format().clone()
    }

    pub fn set_new_text_format(self, tf: TextFormat) {
        self.0.text_spans.borrow_mut().set_default_format(tf);
    }

    pub fn text_format(self, from: usize, to: usize) -> TextFormat {
        if from == to {
            return Default::default();
        }

        // TODO: Convert to byte indices
        self.0.text_spans.borrow().get_text_format(from, to)
    }

    pub fn set_text_format(
        self,
        from: usize,
        to: usize,
        tf: TextFormat,
        context: &mut UpdateContext<'gc>,
    ) {
        // TODO: Convert to byte indices
        self.0
            .text_spans
            .borrow_mut()
            .set_text_format(from, to, &tf);
        self.relayout(context);
    }

    pub fn is_editable(self) -> bool {
        !self.contains_flag(EditTextFlag::READ_ONLY)
    }

    pub fn was_static(self) -> bool {
        self.contains_flag(EditTextFlag::WAS_STATIC)
    }

    pub fn set_editable(self, is_editable: bool) {
        self.set_flag(EditTextFlag::READ_ONLY, !is_editable);
    }

    pub fn is_mouse_wheel_enabled(self) -> bool {
        self.0.mouse_wheel_enabled.get()
    }

    pub fn set_mouse_wheel_enabled(self, is_enabled: bool) {
        self.0.mouse_wheel_enabled.set(is_enabled);
    }

    pub fn is_multiline(self) -> bool {
        self.contains_flag(EditTextFlag::MULTILINE)
    }

    pub fn is_password(self) -> bool {
        self.contains_flag(EditTextFlag::PASSWORD)
    }

    pub fn set_password(self, is_password: bool, context: &mut UpdateContext<'gc>) {
        self.set_flag(EditTextFlag::PASSWORD, is_password);
        self.relayout(context);
    }

    pub fn restrict(self) -> Option<WString> {
        return self.0.restrict.borrow().value().map(Into::into);
    }

    pub fn set_restrict(self, text: Option<&WStr>) {
        self.0.restrict.replace(EditTextRestrict::from(text));
    }

    pub fn set_multiline(self, is_multiline: bool, context: &mut UpdateContext<'gc>) {
        self.set_flag(EditTextFlag::MULTILINE, is_multiline);
        self.relayout(context);
    }

    pub fn is_selectable(self) -> bool {
        !self.contains_flag(EditTextFlag::NO_SELECT)
    }

    pub fn set_selectable(self, is_selectable: bool) {
        self.set_flag(EditTextFlag::NO_SELECT, !is_selectable);
    }

    pub fn is_word_wrap(self) -> bool {
        self.contains_flag(EditTextFlag::WORD_WRAP)
    }

    pub fn set_word_wrap(self, is_word_wrap: bool, context: &mut UpdateContext<'gc>) {
        self.set_flag(EditTextFlag::WORD_WRAP, is_word_wrap);
        self.relayout(context);
    }

    pub fn autosize(self) -> AutoSizeMode {
        self.0.autosize.get()
    }

    pub fn set_autosize(self, asm: AutoSizeMode, context: &mut UpdateContext<'gc>) {
        self.0.autosize.set(asm);
        self.relayout(context);
    }

    pub fn has_background(self) -> bool {
        self.contains_flag(EditTextFlag::HAS_BACKGROUND)
    }

    pub fn set_has_background(self, has_background: bool) {
        self.set_flag(EditTextFlag::HAS_BACKGROUND, has_background);
        self.invalidate_cached_bitmap();
    }

    pub fn background_color(self) -> Color {
        self.0.background_color.get()
    }

    pub fn set_background_color(self, background_color: Color) {
        self.0.background_color.set(background_color);
        self.invalidate_cached_bitmap();
    }

    pub fn has_border(self) -> bool {
        self.contains_flag(EditTextFlag::BORDER)
    }

    pub fn set_has_border(self, has_border: bool) {
        self.set_flag(EditTextFlag::BORDER, has_border);
        self.invalidate_cached_bitmap();
    }

    pub fn border_color(self) -> Color {
        self.0.border_color.get()
    }

    pub fn set_border_color(self, border_color: Color) {
        self.0.border_color.set(border_color);
        self.invalidate_cached_bitmap();
    }

    pub fn condense_white(self) -> bool {
        self.contains_flag(EditTextFlag::CONDENSE_WHITE)
    }

    pub fn set_condense_white(self, condense_white: bool) {
        self.set_flag(EditTextFlag::CONDENSE_WHITE, condense_white);
    }

    pub fn always_show_selection(self) -> bool {
        self.contains_flag(EditTextFlag::ALWAYS_SHOW_SELECTION)
    }

    pub fn set_always_show_selection(self, value: bool) {
        self.set_flag(EditTextFlag::ALWAYS_SHOW_SELECTION, value);
    }

    pub fn is_device_font(self) -> bool {
        !self.contains_flag(EditTextFlag::USE_OUTLINES)
    }

    pub fn set_is_device_font(self, context: &mut UpdateContext<'gc>, is_device_font: bool) {
        self.set_flag(EditTextFlag::USE_OUTLINES, !is_device_font);
        self.relayout(context);
    }

    pub fn font_type(self) -> FontType {
        self.0.font_type()
    }

    pub fn is_html(self) -> bool {
        self.contains_flag(EditTextFlag::HTML)
    }

    pub fn is_effectively_html(self) -> bool {
        self.contains_flag(EditTextFlag::HTML) || self.0.style_sheet.get().is_some()
    }

    pub fn set_is_html(self, is_html: bool) {
        self.set_flag(EditTextFlag::HTML, is_html);
    }

    pub fn style_sheet(self) -> Option<StyleSheet<'gc>> {
        self.0.style_sheet.get().style_sheet()
    }

    pub fn style_sheet_avm1(self) -> Option<Avm1Object<'gc>> {
        if let EditTextStyleSheet::Avm1(object) = self.0.style_sheet.get() {
            Some(object)
        } else {
            None
        }
    }

    pub fn style_sheet_avm2(self) -> Option<Avm2StyleSheetObject<'gc>> {
        if let EditTextStyleSheet::Avm2(style_sheet_object) = self.0.style_sheet.get() {
            Some(style_sheet_object)
        } else {
            None
        }
    }

    pub fn set_style_sheet_avm1(
        self,
        context: &mut UpdateContext<'gc>,
        style_sheet: Option<Avm1Object<'gc>>,
    ) {
        self.set_style_sheet(
            context,
            style_sheet
                .map(EditTextStyleSheet::Avm1)
                .unwrap_or_default(),
        );
    }

    pub fn set_style_sheet_avm2(
        self,
        context: &mut UpdateContext<'gc>,
        style_sheet: Option<Avm2StyleSheetObject<'gc>>,
    ) {
        self.set_is_html(true);
        self.set_style_sheet(
            context,
            style_sheet
                .map(EditTextStyleSheet::Avm2)
                .unwrap_or_default(),
        );
    }

    fn set_style_sheet(
        self,
        context: &mut UpdateContext<'gc>,
        style_sheet: EditTextStyleSheet<'gc>,
    ) {
        unlock!(Gc::write(context.gc(), self.0), EditTextData, style_sheet).set(style_sheet);

        if self.0.style_sheet.get().is_none() {
            self.0.original_html_text.take();
        }

        let original_html_text = self.0.original_html_text.borrow().clone();
        if let Some(html) = original_html_text {
            self.0.parse_html(&html);
        }
        self.relayout(context);
    }

    pub fn is_fte(self) -> bool {
        self.0.is_fte.get()
    }

    pub fn set_is_fte(self, is_fte: bool) {
        self.0.is_fte.set(is_fte);
    }

    pub fn layout_debug_boxes_flag(self, flag: LayoutDebugBoxesFlag) -> bool {
        self.0.layout_debug_boxes_flags.get().contains(flag)
    }

    pub fn set_layout_debug_boxes_flag(self, flag: LayoutDebugBoxesFlag, value: bool) {
        let mut flags = self.0.layout_debug_boxes_flags.get();
        flags.set(flag, value);
        self.0.layout_debug_boxes_flags.set(flags);
    }

    fn set_object(self, value: Option<AvmObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), EditTextData, object).set(value);
    }

    fn set_bound_display_object(self, value: Option<DisplayObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), EditTextData, bound_display_object).set(value);
    }

    /// Returns the matrix for transforming from layout
    /// coordinate space into this object's local space.
    fn layout_to_local_matrix(self) -> Matrix {
        let bounds = self.0.bounds.get();
        Matrix::translate(
            bounds.x_min + Self::GUTTER - Twips::from_pixels(self.0.hscroll.get()),
            bounds.y_min + Self::GUTTER - self.0.vertical_scroll_offset(),
        )
    }

    /// Returns the matrix for transforming from this object's
    /// local space into its layout coordinate space.
    fn local_to_layout_matrix(self) -> Matrix {
        // layout_to_local contains only a translation,
        // no need to inverse the matrix generically.
        let Matrix { tx, ty, .. } = self.layout_to_local_matrix();
        Matrix::translate(-tx, -ty)
    }

    fn local_to_layout(self, local: Point<Twips>) -> Point<Twips> {
        self.local_to_layout_matrix() * local
    }

    pub fn replace_text(
        self,
        from: usize,
        to: usize,
        text: &WStr,
        context: &mut UpdateContext<'gc>,
    ) {
        self.0.text_spans.borrow_mut().replace_text(from, to, text);
        self.relayout(context);
    }

    /// Construct a base text transform for a particular `EditText` span.
    ///
    /// This `text_transform` is separate from and relative to the base
    /// transform that this `EditText` automatically gets by virtue of being a
    /// `DisplayObject`.
    pub fn text_transform(self, color: Color) -> Transform {
        let mut transform: Transform = Default::default();
        transform.color_transform.set_mult_color(color);
        transform
    }

    /// Returns the variable that this text field is bound to.
    pub fn variable(self) -> Option<AvmString<'gc>> {
        self.0.variable.get()
    }

    pub fn set_variable(
        self,
        variable: Option<AvmString<'gc>>,
        activation: &mut Avm1Activation<'_, 'gc>,
    ) {
        // Clear previous binding.
        if let Some(dobj) = self.0.bound_display_object.take() {
            Avm1TextFieldBinding::clear_binding(dobj, self, activation.gc());
        } else {
            activation
                .context
                .unbound_text_fields
                .retain(|&text_field| !DisplayObject::ptr_eq(text_field.into(), self.into()));
        }

        // Setup new binding.
        let text = self.0.shared.initial_text.clone().unwrap_or_default();
        self.set_text(&text, activation.context);

        unlock!(Gc::write(activation.gc(), self.0), EditTextData, variable).set(variable);
        self.try_bind_text_field_variable(activation, true);
    }

    /// Relayout the `EditText`.
    ///
    /// This function operates exclusively with the text-span representation of
    /// the text, and no higher-level representation. Specifically, CSS should
    /// have already been calculated and applied to HTML trees lowered into the
    /// text-span representation.
    pub fn relayout(self, context: &mut UpdateContext<'gc>) {
        let autosize = self.0.autosize.get();
        let is_word_wrap = self.0.flags.get().contains(EditTextFlag::WORD_WRAP);
        let movie = self.0.shared.swf.clone();
        let padding = Self::GUTTER * 2;

        let mut text_spans = self.0.text_spans.borrow_mut();
        if self.0.flags.get().contains(EditTextFlag::PASSWORD) {
            // If the text is a password, hide the text
            text_spans.hide_text();
        } else if text_spans.has_displayed_text() {
            // If it is not a password and has displayed text, we can clear the displayed text
            text_spans.clear_displayed_text();
        }

        // Determine the internal width available for content layout.
        let content_width = if autosize == AutoSizeMode::None || is_word_wrap {
            Some(self.0.bounds.get().width() - padding)
        } else {
            None
        };

        let new_layout = html::lower_from_text_spans(
            &text_spans,
            context,
            movie,
            content_width,
            !self.0.flags.get().contains(EditTextFlag::READ_ONLY),
            is_word_wrap,
            self.0.font_type(),
        );
        drop(text_spans);

        unlock!(Gc::write(context.gc(), self.0), EditTextData, layout).replace(new_layout);
        // reset scroll
        self.0.hscroll.set(0.0);
        self.0.scroll.set(1);

        let text_size = self.0.layout.borrow().text_size();

        let mut autosize_bounds = self.0.bounds.get();
        if autosize != AutoSizeMode::None {
            if !is_word_wrap {
                // The edit text's bounds needs to have the padding baked in.
                let mut width = text_size.width() + padding;
                if !self.0.flags.get().contains(EditTextFlag::READ_ONLY) {
                    // When the field is editable, FP adds 2.5px to add some
                    // space to place the caret.
                    width += Twips::from_pixels(2.5);
                }
                let new_x = match autosize {
                    AutoSizeMode::Left => autosize_bounds.x_min,
                    AutoSizeMode::Center => {
                        (autosize_bounds.x_min + autosize_bounds.x_max - width) / 2
                    }
                    AutoSizeMode::Right => autosize_bounds.x_max - width,
                    AutoSizeMode::None => unreachable!(),
                };
                autosize_bounds.x_min = new_x;
                autosize_bounds.set_width(width);
            }
            let height = text_size.height() + padding;
            autosize_bounds.set_height(height);
        }
        self.0.autosize_lazy_bounds.set(Some(autosize_bounds));
        self.invalidate_cached_bitmap();
    }

    /// Apply lazily calculated autosize bounds.
    ///
    /// They should be applied only in specific places, as they influence
    /// the behavior of other actions performed on the text field.
    ///
    /// For instance, consider the following code.
    ///
    /// ```as3
    /// var text = new TextField();
    /// text.text = "Hello World";
    ///
    /// text.autoSize = "left";
    /// // The autosize bounds cannot be applied here, as otherwise
    /// // the following wordWrap and autoSize would not work.
    /// text.wordWrap = true;
    /// text.autoSize = "right";
    ///
    /// // The autosize bounds have to be applied here, as we're
    /// // accessing x and othrwise we would have read a wrong value.
    /// trace(text.x);
    /// ```
    pub fn apply_autosize_bounds(self) {
        if let Some(bounds) = self.0.autosize_lazy_bounds.take() {
            self.0.bounds.set(bounds);
            // Note: We do not have to invalidate cache here.
            //   Cache has already been invalidated on relayout, and
            //   we will apply this anyway before render.
        }
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, _context: &mut UpdateContext<'gc>) -> (Twips, Twips) {
        let text_size = self.0.layout.borrow().text_size();
        (text_size.width(), text_size.height())
    }

    /// How far the text can be scrolled right, in pixels.
    pub fn maxhscroll(self) -> f64 {
        // word-wrapped text can't be scrolled
        if self.0.flags.get().contains(EditTextFlag::WORD_WRAP) {
            return 0.0;
        }

        let layout = self.0.layout.borrow();
        let mut text_width = layout.text_size().width();
        let window_width = (self.0.bounds.get().width() - Self::GUTTER * 2).max(Twips::ZERO);

        if !self.0.flags.get().contains(EditTextFlag::READ_ONLY) {
            // input fields get extra space at the end
            text_width += window_width / 4;
        }

        (text_width - window_width)
            .trunc_to_pixel()
            .to_pixels()
            .max(0.0)
    }

    /// How many lines the text can be scrolled down
    pub fn maxscroll(self) -> usize {
        // FIXME [KJ] The following logic is yet inaccurate
        //   for some input fields and negative leading.
        //   Might be related to text height calculation.
        let layout = self.0.layout.borrow();
        let lines = layout.lines();

        if lines.is_empty() {
            return 1;
        }

        let text_height = layout.text_size().height();
        let window_height = self.0.bounds.get().height() - Self::GUTTER * 2;

        // That's the y coordinate where the fully scrolled window begins.
        // We have to find a line that's below this coordinate.
        let target = text_height - window_height;

        // TODO Use binary search here
        let line = lines.iter().find(|&l| l.offset_y() >= target);
        if let Some(line) = line {
            line.index() + 1
        } else {
            // I don't know how this could happen, so return the limit
            lines.last().unwrap().index() + 1
        }
    }

    /// The lowest visible line of text
    pub fn bottom_scroll(self) -> usize {
        let layout = self.0.layout.borrow();
        let lines = layout.lines();

        if lines.is_empty() {
            return 1;
        }

        let scroll_offset = lines
            .get(self.0.scroll.get() - 1)
            .map_or(Twips::ZERO, |l| l.offset_y());
        let target = self.0.bounds.get().height() + scroll_offset - Self::GUTTER * 2;

        // TODO Use binary search here
        // Line before first line with extent greater than bounds.height() + line "scroll"'s offset
        let too_far = lines.iter().find(|&l| l.extent_y() > target);
        if let Some(line) = too_far {
            line.index().max(1)
        } else {
            // all lines are visible
            lines.last().unwrap().index() + 1
        }
    }

    /// Returns the selection, but takes into account whether the selection should be rendered.
    fn visible_selection(self) -> Option<TextSelection> {
        let selection = self.0.selection.get()?;
        // TODO: Remove this #[allow] once Rust 1.94 is released.
        // Clippy 0.1.94+ (PR #16286) no longer fires collapsible_else_if when both
        // branches contain if-else expressions, recognizing the parallel structure.
        #[allow(clippy::collapsible_else_if)]
        if selection.is_caret() {
            if self.has_focus() && !self.0.flags.get().contains(EditTextFlag::READ_ONLY) {
                Some(selection)
            } else {
                None
            }
        } else {
            if self.has_focus() || self.always_show_selection() {
                Some(selection)
            } else {
                None
            }
        }
    }

    fn render_debug_boxes(
        self,
        context: &mut RenderContext<'_, 'gc>,
        flags: LayoutDebugBoxesFlag,
        layout: &Layout<'gc>,
    ) {
        if flags.contains(LayoutDebugBoxesFlag::CHAR) {
            for i in 0..self.text().len() {
                if let Some(bounds) = layout.char_bounds(i) {
                    context.draw_rect_outline(Color::MAGENTA, bounds, Twips::ONE_PX);
                }
            }
        }
        if flags.contains(LayoutDebugBoxesFlag::BOX) {
            for lbox in layout.boxes_iter() {
                context.draw_rect_outline(Color::RED, lbox.bounds().into(), Twips::ONE_PX);
            }
        }
        if flags.contains(LayoutDebugBoxesFlag::LINE) {
            for line in layout.lines() {
                context.draw_rect_outline(Color::BLUE, line.bounds().into(), Twips::ONE_PX);
            }
        }
        if flags.contains(LayoutDebugBoxesFlag::TEXT) {
            context.draw_rect_outline(Color::GREEN, layout.bounds().into(), Twips::ONE_PX);
        }
    }

    /// Render lines according to the given procedure.
    ///
    /// This skips invisible lines.
    fn render_lines<F>(self, context: &mut RenderContext<'_, 'gc>, mut f: F)
    where
        F: FnMut(&mut RenderContext<'_, 'gc>, &LayoutLine<'gc>),
    {
        // Skip lines that are off-screen.
        let lines_to_skip = self.scroll().saturating_sub(1);
        for line in self.0.layout.borrow().lines().iter().skip(lines_to_skip) {
            f(context, line);
        }
    }

    /// Render the visible text along with selection and the caret.
    fn render_text(
        self,
        context: &mut RenderContext<'_, 'gc>,
        render_state: &mut EditTextRenderState,
    ) {
        self.render_selection_background(context);
        self.render_lines(context, |context, line| {
            self.render_layout_line(context, line, render_state);
        });
    }

    /// Render the black selection background.
    fn render_selection_background(self, context: &mut RenderContext<'_, 'gc>) {
        let Some(selection) = self.visible_selection() else {
            return;
        };
        if selection.is_caret() {
            return;
        }

        let (start, end) = (selection.start(), selection.end());

        self.render_lines(context, |context, line| {
            self.render_selection_background_for_line(context, line, start, end)
        });
    }

    fn render_selection_background_for_line(
        self,
        context: &mut RenderContext<'_, 'gc>,
        line: &LayoutLine<'gc>,
        start: usize,
        end: usize,
    ) {
        let local_start = start.clamp(line.start(), line.end());
        let local_end = end.clamp(line.start(), line.end());

        if local_start >= local_end {
            // No selection in this line
            return;
        }

        let line_bounds = line.bounds();

        // If the selection ends within this line, the background
        // is not drawn over leading.
        let leading = if local_end == end {
            Twips::ZERO
        } else {
            line.leading()
        };

        let x_start = line
            .char_x_bounds(local_start)
            .map(|b| b.0)
            .unwrap_or_else(|| line_bounds.offset_x());
        let x_end = line
            .char_x_bounds(local_end - 1)
            .map(|b| b.1)
            .unwrap_or_else(|| line_bounds.extent_x());

        let width = x_end - x_start;
        let height = line_bounds.height() + leading;

        let color = if self.has_focus() {
            Color::BLACK
        } else {
            Color::GRAY
        };
        let selection_box = context.transform_stack.transform().matrix
            * Matrix::create_box(
                width.to_pixels() as f32,
                height.to_pixels() as f32,
                x_start,
                line_bounds.origin().y(),
            );
        context.commands.draw_rect(color, selection_box);
    }

    fn render_layout_line(
        self,
        context: &mut RenderContext<'_, 'gc>,
        line: &LayoutLine<'gc>,
        render_state: &mut EditTextRenderState,
    ) {
        let max_descent = line.descent();
        for layout_box in line.boxes_iter() {
            self.render_layout_box(context, layout_box, render_state, max_descent);
        }
    }

    /// Render a layout box, plus its children.
    fn render_layout_box(
        self,
        context: &mut RenderContext<'_, 'gc>,
        lbox: &LayoutBox<'gc>,
        render_state: &mut EditTextRenderState,
        max_descent: Twips,
    ) {
        let origin = lbox.bounds().origin();

        // If text's top is under the textbox's bottom, skip drawing.
        // TODO: FP actually skips drawing a line as soon as its bottom is under the textbox;
        //   Current logic is conservative for safety (and even of this I'm not 100% sure).
        //   (maybe we could cull-before-render all glyphs, thus removing the need for masking?)
        // [KJ] FP always displays the first visible line (sometimes masked, sometimes sticking out of bounds),
        //      culls any other line which is not fully visible; masking is always used for left/right bounds
        // TODO: also cull text that's simply out of screen, just like we cull whole DOs in render_self().
        if origin.y() + Self::GUTTER - self.0.vertical_scroll_offset()
            > self.0.bounds.get().height()
        {
            return;
        }

        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(origin.x(), origin.y()),
            ..Default::default()
        });

        let visible_selection = self.visible_selection();

        let caret = if let LayoutContent::Text { start, end, .. } = &lbox.content() {
            if let Some(visible_selection) = visible_selection {
                let text_len = self.0.text_spans.borrow().text().len();
                if visible_selection.is_caret()
                    && !self.0.flags.get().contains(EditTextFlag::READ_ONLY)
                    && visible_selection.start() >= *start
                    && (visible_selection.end() < *end || *end == text_len)
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
            lbox.as_renderable_text(self.0.text_spans.borrow().displayed_text())
        {
            let baseline = font.get_baseline_for_height(params.height());
            let descent = font.get_descent_for_height(params.height());
            let caret_height = baseline + descent;
            let mut caret_x = Twips::ZERO;
            font.evaluate(
                text,
                self.text_transform(color),
                params,
                &mut |pos, transform, glyph, advance, x| {
                    if glyph.renderable(context) {
                        // If it's highlighted, override the color.
                        if matches!(visible_selection, Some(visible_selection) if visible_selection.contains(start + pos)) {
                            // Set text color to white
                            context.transform_stack.push(&Transform {
                                matrix: transform.matrix,
                                color_transform: ColorTransform::IDENTITY,
                                perspective_projection: transform.perspective_projection,
                            });
                        } else {
                            context.transform_stack.push(transform);
                        }
                        glyph.render(context);
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
                self.render_caret(context, caret_x, caret_height, color, render_state);
            }

            if let LayoutContent::Text {
                underline: true, ..
            } = lbox.content()
            {
                // Draw underline
                let underline_y = baseline + (max_descent / 2);
                let underline_width = lbox.bounds().width();
                self.render_underline(context, underline_width, underline_y, color);
            }
        }

        if let Some(drawing) = lbox.as_renderable_drawing() {
            drawing.render(context);
        }

        context.transform_stack.pop();
    }

    fn render_caret(
        self,
        context: &mut RenderContext<'_, 'gc>,
        x: Twips,
        height: Twips,
        color: Color,
        render_state: &mut EditTextRenderState,
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

        // We have to draw the caret outside of the text mask.
        render_state.draw_caret_command = Some(RenderCommand::DrawLine {
            color,
            matrix: caret,
        });
    }

    fn render_underline(
        self,
        context: &mut RenderContext<'_, 'gc>,
        width: Twips,
        y: Twips,
        color: Color,
    ) {
        let mut underline = context.transform_stack.transform().matrix
            * Matrix::create_box_with_rotation(width.to_pixels() as f32, 1.0, 0.0, Twips::ZERO, y);

        let pixel_snapping = EditTextPixelSnapping::new(context.stage.quality());
        pixel_snapping.apply(&mut underline);

        context.commands.draw_line(color, underline);
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
        let Some(variable_path) = self.variable() else {
            // No variable for this text field; success by default
            return true;
        };

        // Any previous binding should have been cleared.
        debug_assert!(self.0.bound_display_object.get().is_none());

        let Some(mut parent) = self.avm1_parent() else {
            return false;
        };
        while parent.as_avm1_button().is_some() {
            let Some(p) = parent.avm1_parent() else {
                return false;
            };
            parent = p;
        }

        let mut bound = false;
        activation.run_with_child_frame_for_display_object(
            "[Text Field Binding]",
            parent,
            self.movie().version(),
            |activation| {
                if let Ok(Some((object, property))) =
                    activation.resolve_variable_path(parent, &variable_path)
                {
                    let property = AvmString::new(activation.gc(), property);

                    // If this text field was just created, we immediately propagate the text to the variable (or vice versa).
                    if set_initial_value {
                        // If the property exists on the object, we overwrite the text with the property's value.
                        if object.has_property(activation, property) {
                            let value = object.get(property, activation).unwrap();
                            self.set_html_text(
                                &value
                                    .coerce_to_string(activation)
                                    .unwrap_or_else(|_| istr!("")),
                                activation.context,
                            );
                        } else {
                            // Otherwise, we initialize the property with the text field's text, if it's non-empty.
                            // Note that HTML text fields are often initialized with an empty <p> tag, which is not considered empty.
                            let text = self.text();
                            if !text.is_empty() {
                                let _ = object.set(
                                    property,
                                    AvmString::new(activation.gc(), self.text()).into(),
                                    activation,
                                );
                            }
                        }
                    }

                    if let Some(dobj) = object.as_display_object() {
                        self.set_bound_display_object(Some(dobj), activation.gc());
                        let binding = Avm1TextFieldBinding {
                            text_field: self,
                            variable_name: property,
                        };
                        binding.register_binding(dobj, activation.gc());
                        bound = true;
                    }
                }
            },
        );
        bound
    }

    /// Unsets a bound display object from this text field.
    /// Does not change the unbound text field list.
    /// Caller is responsible for adding this text field to the unbound list, if necessary.
    pub fn clear_bound_display_object(self, context: &mut UpdateContext<'gc>) {
        self.set_bound_display_object(None, context.gc());
    }

    /// Propagates a text change to the bound display object.
    ///
    pub fn propagate_text_binding(self, activation: &mut Avm1Activation<'_, 'gc>) {
        if !self.contains_flag(EditTextFlag::FIRING_VARIABLE_BINDING) {
            self.set_flag(EditTextFlag::FIRING_VARIABLE_BINDING, true);
            if let Some(variable_path) = self.variable()
                && let Ok(Some((object, property))) =
                    activation.resolve_variable_path(self.avm1_parent().unwrap(), &variable_path)
            {
                // Note that this can call virtual setters, even though the opposite direction won't work
                // (virtual property changes do not affect the text field)
                activation.run_with_child_frame_for_display_object(
                    "[Propagate Text Binding]",
                    self.avm1_parent().unwrap(),
                    self.movie().version(),
                    |activation| {
                        let property = AvmString::new(activation.gc(), property);
                        let _ = object.set(
                            property,
                            AvmString::new(activation.gc(), self.html_text()).into(),
                            activation,
                        );
                    },
                );
            }
            self.set_flag(EditTextFlag::FIRING_VARIABLE_BINDING, false);
        }
    }

    pub fn selection(self) -> Option<TextSelection> {
        self.0.selection.get()
    }

    pub fn set_selection(self, selection: Option<TextSelection>) {
        let old_selection = self.0.selection.get();
        if let Some(mut selection) = selection {
            selection.clamp(self.0.text_spans.borrow().text().len());
            self.0.selection.set(Some(selection));
        } else {
            self.0.selection.set(None);
        }

        if old_selection != self.0.selection.get() {
            self.invalidate_cached_bitmap();
        }
    }

    /// Calculate and return the [`TextSelection`] at the given position
    /// using the given selection mode.
    fn calculate_selection_at(self, position: usize, mode: TextSelectionMode) -> TextSelection {
        match mode {
            TextSelectionMode::Character => TextSelection::for_position(position),
            TextSelectionMode::Word => {
                let from = self.find_prev_word_boundary(position, true);
                let to = self.find_next_word_boundary(position, true);
                TextSelection::for_range(from, to)
            }
            TextSelectionMode::Line => {
                let from = self.find_prev_line_boundary(position);
                let to = self.find_next_line_boundary(position);
                TextSelection::for_range(from, to)
            }
        }
    }

    pub fn reset_selection_blinking(self) {
        if let Some(mut selection) = self.0.selection.get() {
            selection.reset_blinking();
            self.0.selection.set(Some(selection));
        }
    }

    pub fn spans(&self) -> Ref<'_, FormatSpans> {
        self.0.text_spans.borrow()
    }

    pub fn layout(&self) -> Ref<'_, Layout<'gc>> {
        self.0.layout.borrow()
    }

    pub fn render_settings(self) -> TextRenderSettings {
        self.0.render_settings.get()
    }

    pub fn set_render_settings(self, settings: TextRenderSettings) {
        self.0.render_settings.set(settings)
    }

    pub fn hscroll(self) -> f64 {
        self.0.hscroll.get()
    }

    pub fn set_hscroll(self, hscroll: f64) {
        self.0.hscroll.set(hscroll);
        self.invalidate_cached_bitmap();
    }

    pub fn scroll(self) -> usize {
        self.0.scroll.get()
    }

    /// Returns `true` when scroll has been modified.
    pub fn set_scroll(self, scroll: f64) -> bool {
        // derived experimentally. Not exact: overflows somewhere above 767100486418432.9
        // Checked in SWF 6, AVM1. Same in AVM2.
        const SCROLL_OVERFLOW_LIMIT: f64 = 767100486418433.0;
        let scroll_lines = if scroll.is_nan() || scroll < 0.0 || scroll >= SCROLL_OVERFLOW_LIMIT {
            1
        } else {
            scroll as usize
        };
        let clamped = scroll_lines.clamp(1, self.maxscroll());
        if self.0.scroll.replace(clamped) == clamped {
            false
        } else {
            self.invalidate_cached_bitmap();
            true
        }
    }

    pub fn max_chars(self) -> i32 {
        self.0.max_chars.get()
    }

    pub fn set_max_chars(self, value: i32) {
        self.0.max_chars.set(value);
    }

    /// Map the position on the screen to caret index.
    ///
    /// This method is used exclusively for placing a caret inside text.
    /// It implements the Flash Player's behavior of placing a caret.
    /// Characters are divided in half, the last line is extended, etc.
    pub fn screen_position_to_index(self, position: Point<Twips>) -> Option<usize> {
        let position = self.global_to_local(position)?;
        let position = self.local_to_layout(position);

        // TODO We can use binary search for both y and x here

        // First determine which line of text is the closest match to the Y position...
        let layout = self.0.layout.borrow();
        let line_index = layout
            .find_line_index_by_y(position.y)
            .unwrap_or_else(|i| i);
        let line = layout.lines().get(line_index)?;

        // ...then find the box within that line that is the closest match to the X position.
        let mut closest_layout_box: Option<&LayoutBox<'gc>> = None;
        for layout_box in line.boxes_iter() {
            if layout_box.is_text_box() {
                if position.x >= layout_box.bounds().offset_x() || closest_layout_box.is_none() {
                    closest_layout_box = Some(layout_box);
                } else {
                    break;
                }
            }
        }

        if let Some(layout_box) = closest_layout_box {
            let origin = layout_box.bounds().origin();
            let mut matrix = Matrix::translate(origin.x(), origin.y());
            matrix = matrix.inverse().expect("Invertible layout matrix");
            let local_position = matrix * position;

            if let Some((text, _tf, font, params, color)) =
                layout_box.as_renderable_text(self.0.text_spans.borrow().text())
            {
                let mut result = 0;
                font.evaluate(
                    text,
                    self.text_transform(color),
                    params,
                    &mut |pos, _transform, _glyph, advance, x| {
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
        let max_chars = self.0.max_chars.get();
        if max_chars == 0 {
            usize::MAX
        } else {
            let text_len = self.0.text_spans.borrow().text().len() as i32;
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
        context: &mut UpdateContext<'gc>,
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
        context: &mut UpdateContext<'gc>,
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
                self.text_input(Self::INPUT_NEWLINE.to_string(), context);
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
                self.set_selection(Some(TextSelection::for_position(new_pos)));
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
                self.set_selection(Some(TextSelection::for_position(new_pos)));
            }
            TextControlCode::SelectLeft
            | TextControlCode::SelectLeftWord
            | TextControlCode::SelectLeftLine
            | TextControlCode::SelectLeftDocument => {
                if selection.to > 0 {
                    let new_pos = self.find_new_position(control_code, selection.to);
                    self.set_selection(Some(TextSelection::for_range(selection.from, new_pos)));
                }
            }
            TextControlCode::SelectRight
            | TextControlCode::SelectRightWord
            | TextControlCode::SelectRightLine
            | TextControlCode::SelectRightDocument => {
                if selection.to < self.text().len() {
                    let new_pos = self.find_new_position(control_code, selection.to);
                    self.set_selection(Some(TextSelection::for_range(selection.from, new_pos)))
                }
            }
            TextControlCode::SelectAll => {
                self.set_selection(Some(TextSelection::for_range(0, self.text().len())));
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

                self.text_input(text, context);
            }
            TextControlCode::Cut => {
                let text = &self.text()[selection.start()..selection.end()];
                context.ui.set_clipboard_content(text.to_string());

                self.replace_text(selection.start(), selection.end(), WStr::empty(), context);
                if is_selectable {
                    self.set_selection(Some(TextSelection::for_position(selection.start())));
                } else {
                    self.set_selection(Some(TextSelection::for_position(self.text().len())));
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
                self.set_selection(Some(TextSelection::for_position(selection.start())));
                changed = true;
            }
            TextControlCode::Backspace | TextControlCode::BackspaceWord => {
                // Backspace with caret
                if selection.start() > 0 {
                    // Delete previous character(s)
                    let start = self.find_new_position(control_code, selection.start());
                    self.replace_text(start, selection.start(), WStr::empty(), context);
                    self.set_selection(Some(TextSelection::for_position(start)));
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
                    self.reset_selection_blinking();
                    changed = true;
                }
            }
        }
        if changed {
            let mut activation = Avm1Activation::from_nothing(
                context,
                ActivationIdentifier::root("[Propagate Text Binding]"),
                self.into(),
            );
            self.propagate_text_binding(&mut activation);
            self.on_changed(&mut activation);
        }
    }

    pub fn ime(self, event: ImeEvent, context: &mut UpdateContext<'gc>) {
        match event {
            ImeEvent::Preedit(text, _) if text.is_empty() => self.ensure_ime_finished(context),
            ImeEvent::Preedit(text_utf8, cursor_utf8) => {
                let ime_data = self.ensure_ime_started(context);

                let text: WString = WString::from_utf8(&text_utf8);

                let cursor = cursor_utf8.map(|(from, to)| {
                    let to_utf8 = WStrToUtf8::new(&text);
                    (
                        to_utf8.utf16_index(from).unwrap_or_else(|| text.len()),
                        to_utf8.utf16_index(to).unwrap_or_else(|| text.len()),
                    )
                });

                let ImeData {
                    ime_start: old_ime_start,
                    ime_end: old_ime_end,
                    ..
                } = ime_data;

                self.replace_text(old_ime_start, old_ime_end, &text, context);

                self.0.ime_data.replace(Some(ImeData {
                    ime_start: old_ime_start,
                    ime_end: old_ime_start + text.len(),
                    text: text_utf8,
                }));

                let new_selection = cursor.map(|(from, to)| {
                    TextSelection::for_range(old_ime_start + from, old_ime_start + to)
                });
                self.set_selection(new_selection);
            }
            ImeEvent::Commit(text) => self.text_input(text, context),
        };
    }

    fn ensure_ime_started(self, context: &mut UpdateContext<'gc>) -> ImeData {
        if let Some(ime_data) = self.0.ime_data.borrow().clone() {
            return ime_data;
        }

        let selection = self.selection().unwrap_or_else(|| {
            TextSelection::for_position(self.0.text_spans.borrow().text().len())
        });
        self.replace_text(selection.start(), selection.end(), WStr::empty(), context);

        let ime_data = ImeData {
            ime_start: selection.start(),
            ime_end: selection.start(),
            text: String::new(),
        };
        self.0.ime_data.replace(Some(ime_data.clone()));
        ime_data
    }

    fn ensure_ime_finished(self, context: &mut UpdateContext<'gc>) {
        let Some(ImeData {
            ime_start, ime_end, ..
        }) = *self.0.ime_data.borrow()
        else {
            return;
        };

        self.replace_text(ime_start, ime_end, WStr::empty(), context);
        self.set_selection(Some(TextSelection::for_position(ime_start)));
        self.0.ime_data.take();
    }

    fn ensure_ime_committed(self, context: &mut UpdateContext<'gc>) {
        let Some(ImeData { text, .. }) = self.0.ime_data.borrow().clone() else {
            return;
        };

        self.ensure_ime_finished(context);
        self.text_input(text, context);
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
            | TextControlCode::DeleteWord => self.find_next_word_boundary(current_pos, false),
            TextControlCode::SelectLeftWord
            | TextControlCode::MoveLeftWord
            | TextControlCode::BackspaceWord => self.find_prev_word_boundary(current_pos, false),
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

    /// Find the nearest word boundary before (or exceptionally at) `pos`,
    /// which is applicable for selection.
    ///
    /// When `stop_on_space` is true, `pos` will be returned if there's space before it.
    ///
    /// This algorithm is based on [UAX #29](https://unicode.org/reports/tr29/).
    fn find_prev_word_boundary(self, pos: usize, stop_on_space: bool) -> usize {
        let head = &self.text()[..pos];
        if stop_on_space && head.ends_with(ruffle_wstr::utils::swf_is_whitespace) {
            return pos;
        }
        let to_utf8 = WStrToUtf8::new(head);
        to_utf8
            .to_utf8_lossy()
            .split_word_bound_indices()
            .rev()
            .find(|(_, span)| !span.trim().is_empty())
            .map(|(position, _)| position)
            .and_then(|utf8_index| to_utf8.utf16_index(utf8_index))
            .unwrap_or(0)
    }

    /// Find the nearest word boundary after (or exceptionally at) `pos`,
    /// which is applicable for selection.
    ///
    /// When `stop_on_space` is true, `pos` will be returned if there's space after it.
    ///
    /// This algorithm is based on [UAX #29](https://unicode.org/reports/tr29/).
    fn find_next_word_boundary(self, pos: usize, stop_on_space: bool) -> usize {
        let tail = &self.text()[pos..];
        if stop_on_space && tail.starts_with(ruffle_wstr::utils::swf_is_whitespace) {
            return pos;
        }
        let to_utf8 = WStrToUtf8::new(tail);
        to_utf8
            .to_utf8_lossy()
            .split_word_bound_indices()
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

    pub fn text_input(self, text: String, context: &mut UpdateContext<'gc>) {
        if !self.is_editable() || self.available_chars() == 0 {
            return;
        }

        let text = if self.is_multiline() {
            text
        } else {
            text.replace(&Self::ANY_NEWLINE[..], "")
        };

        // TODO We need to test it with fonts that provide such characters.
        //   It's possible this is just a case of
        //   "don't input a character without a glyph".
        let text: String = text
            .chars()
            .filter(|ch| !ch.is_control() || Self::ANY_NEWLINE.contains(ch))
            .collect();

        if text.is_empty() {
            return;
        }

        let text = WString::from_utf8(&text);

        let Some(selection) = self.selection() else {
            return;
        };

        let filtered_text = self.0.restrict.borrow().filter_allowed(&text);

        if let Some(target) = self.object2() {
            let character_string =
                AvmString::new(context.gc(), text.replace(b'\r', WStr::from_units(b"\n")));

            let mut activation = Avm2Activation::from_nothing(context);
            let text_evt = Avm2EventObject::text_event(
                &mut activation,
                "textInput",
                character_string,
                true,
                true,
            );
            Avm2::dispatch_event(activation.context, text_evt, target.into());

            if text_evt.event().is_cancelled() {
                return;
            }
        }

        let mut text = filtered_text.as_wstr();
        if text.len() > self.available_chars() {
            text = &text[0..self.available_chars()];
        }

        self.replace_text(selection.start(), selection.end(), text, context);
        let new_pos = selection.start() + text.len();
        self.set_selection(Some(TextSelection::for_position(new_pos)));

        let mut activation = Avm1Activation::from_nothing(
            context,
            ActivationIdentifier::root("[Propagate Text Binding]"),
            self.into(),
        );
        self.propagate_text_binding(&mut activation);
        self.on_changed(&mut activation);
    }

    fn initialize_as_broadcaster(self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Some(object) = self.object1() {
            activation
                .context
                .avm1
                .broadcaster_functions(activation.swf_version())
                .initialize(
                    &activation.context.strings,
                    object,
                    activation.prototypes().array,
                );

            if let Ok(Avm1Value::Object(listeners)) = object.get(istr!("_listeners"), activation) {
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

    fn on_changed(self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Some(object) = self.object1() {
            let _ = object.call_method(
                istr!("broadcastMessage"),
                &[istr!("onChanged").into(), object.into()],
                activation,
                ExecutionReason::Special,
            );
        } else if let Some(object) = self.object2() {
            let change_evt = Avm2EventObject::bare_event(
                activation.context,
                "change",
                true,  /* bubbles */
                false, /* cancelable */
            );
            Avm2::dispatch_event(activation.context, change_evt, object.into());
        }
    }

    fn on_scroller(self, activation: &mut Avm1Activation<'_, 'gc>) {
        if let Some(object) = self.object1() {
            let _ = object.call_method(
                istr!("broadcastMessage"),
                &[istr!("onScroller").into(), object.into()],
                activation,
                ExecutionReason::Special,
            );
        }
        //TODO: Implement this for Avm2
    }

    /// Construct the text field's AVM1 representation.
    fn construct_as_avm1_object(self, context: &mut UpdateContext<'gc>) {
        if self.0.object.get().is_none() {
            let object = Avm1Object::new_with_native(
                &context.strings,
                Some(context.avm1.prototypes(self.swf_version()).text_field),
                Avm1NativeObject::EditText(self),
            );

            self.set_object(Some(object.into()), context.gc());
        }

        Avm1::run_with_stack_frame_for_display_object(self.into(), context, |activation| {
            // If this text field has a variable set, initialize text field binding.
            if !self.try_bind_text_field_variable(activation, true) {
                activation.context.unbound_text_fields.push(self);
            }
            // People can bind to properties of TextFields the same as other display objects.
            Avm1TextFieldBinding::bind_variables(activation);

            self.initialize_as_broadcaster(activation);
        });
    }

    /// Construct the text field's AVM2 representation.
    fn construct_as_avm2_object(
        self,
        context: &mut UpdateContext<'gc>,
        display_object: DisplayObject<'gc>,
    ) {
        let class_object = self
            .0
            .class
            .get()
            .unwrap_or_else(|| context.avm2.classes().textfield);

        let mut activation = Avm2Activation::from_nothing(context);

        match Avm2StageObject::for_display_object_childless(
            &mut activation,
            display_object,
            class_object,
        ) {
            Ok(object) => {
                self.set_object(Some(object.into()), context.gc());
            }
            Err(err) => {
                Avm2::uncaught_error(
                    &mut activation,
                    Some(self.into()),
                    err,
                    "Error running AVM2 construction for dynamic text",
                );
            }
        }
    }

    /// Count the number of lines in the text box's layout.
    pub fn layout_lines(self) -> usize {
        self.0.layout.borrow().lines().len()
    }

    /// Calculate the layout metrics.
    ///
    /// Returns `None` if there is not enough data
    /// about the layout to calculate metrics with.
    pub fn layout_metrics(self) -> Option<LayoutMetrics> {
        let layout = &self.0.layout.borrow();

        let boxes = layout.boxes_iter();
        let union_bounds = layout.bounds();

        let mut first_font_set = None;
        let mut first_format = None;
        for layout_box in boxes {
            match layout_box.content() {
                LayoutContent::Text {
                    font_set,
                    text_format,
                    ..
                }
                | LayoutContent::Bullet {
                    font_set,
                    text_format,
                    ..
                } => {
                    first_font_set = Some(font_set);
                    first_format = Some(text_format);
                    break;
                }
                LayoutContent::Drawing { .. } => {}
            }
        }

        let font_set = first_font_set?;
        let text_format = first_format?;
        let size = Twips::from_pixels(text_format.size?);
        let ascent = font_set.get_baseline_for_height(size);
        let descent = font_set.get_descent_for_height(size);
        let leading = Twips::from_pixels(text_format.leading?);

        Some(LayoutMetrics {
            ascent,
            descent,
            leading,
            width: union_bounds.width(),
            height: union_bounds.height() + descent + leading,
            x: union_bounds.offset_x() + Self::GUTTER,
        })
    }

    pub fn line_metrics(self, line: usize) -> Option<LayoutMetrics> {
        let layout = &self.0.layout.borrow();
        let line = layout.lines().get(line)?;
        let bounds = line.bounds();

        Some(LayoutMetrics {
            ascent: line.ascent(),
            descent: line.descent(),
            leading: line.leading(),
            width: bounds.width(),
            height: bounds.height() + line.leading(),
            x: bounds.offset_x() + Self::GUTTER,
        })
    }

    pub fn line_length(self, line: usize) -> Option<usize> {
        Some(self.0.layout.borrow().lines().get(line)?.len())
    }

    pub fn line_text(self, line: usize) -> Option<WString> {
        let layout = self.0.layout.borrow();
        let line = layout.lines().get(line)?;
        let text_spans = self.0.text_spans.borrow();
        let line_text = text_spans.text().slice(line.text_range())?;
        Some(WString::from_wstr(line_text))
    }

    pub fn line_offset(self, line: usize) -> Option<usize> {
        let layout = self.0.layout.borrow();
        let line = layout.lines().get(line)?;
        let first_box = line.boxes_iter().next()?;
        Some(first_box.start())
    }

    /// Returns the index of the line that is at the given position.
    ///
    /// It returns `None` when there's no line at the given position,
    /// with the exception that positions below the last line will
    /// return the index of the last line.
    pub fn line_index_at_point(self, position: Point<Twips>) -> Option<usize> {
        // Check bounds
        let bounds = self.0.bounds.get().grow(-Self::GUTTER);
        if !bounds.contains(position) {
            return None;
        }

        let position = self.local_to_layout(position);

        Some(
            self.0
                .layout
                .borrow()
                .find_line_index_by_y(position.y)
                .unwrap_or_else(|i| i),
        )
    }

    /// Returns the index of the character that is at the given position.
    ///
    /// It returns `None` when there's no character at the given position.
    /// It takes into account various quirks of Flash Player:
    ///  1. It will return the index of the newline when `x`
    ///     is zero and the line is empty.
    ///  2. It assumes (exclusive, inclusive) bounds.
    ///  3. Positions with `y` below the last line will behave
    ///     the same way as at the last line.
    pub fn char_index_at_point(self, position: Point<Twips>) -> Option<usize> {
        let line_index = self.line_index_at_point(position)?;

        let layout = self.0.layout.borrow();
        let line = &layout.lines()[line_index];

        // KJ: It's a bug in FP, it doesn't take into account horizontal
        // scroll, but it does take into account vertical scroll.
        // See https://github.com/airsdk/Adobe-Runtime-Support/issues/2315
        // I guess we'll have to take scrollH into account here when
        // we start supporting Harman runtimes.
        let x = position.x - Self::GUTTER;

        // Yes, this will return the index of the newline when the line is empty.
        // Yes, that's how Flash Player does it.
        if x == Twips::ZERO {
            return Some(line.start());
        }

        // TODO Use binary search here when possible
        for ch in line.start()..line.end() {
            let bounds = line.char_x_bounds(ch);
            let Some((a, b)) = bounds else {
                continue;
            };

            if a < x && x <= b {
                return Some(ch);
            }
        }

        None
    }

    pub fn line_index_of_char(self, index: usize) -> Option<usize> {
        self.0.layout.borrow().find_line_index_by_position(index)
    }

    pub fn paragraph_start_index_at(self, mut index: usize) -> Option<usize> {
        let text = self.text();

        // Note that the index may equal the text length
        if index > text.len() {
            return None;
        }

        while index > 0 && !string_utils::swf_is_newline(text.at(index - 1)) {
            index -= 1;
        }

        Some(index)
    }

    pub fn paragraph_length_at(self, mut index: usize) -> Option<usize> {
        let start_index = self.paragraph_start_index_at(index)?;
        let text = self.text();
        let length = text.len();

        // When the index is equal to the text length,
        // FP simulates a character at that point and returns
        // the length of the last paragraph plus one.
        if index == length {
            return Some(1 + length - start_index);
        }

        while index < length && !string_utils::swf_is_newline(text.at(index)) {
            index += 1;
        }

        // The trailing newline also counts to the length
        if index < length && string_utils::swf_is_newline(text.at(index)) {
            index += 1;
        }

        Some(index - start_index)
    }

    pub fn char_bounds(self, position: usize) -> Option<Rectangle<Twips>> {
        let layout = self.0.layout.borrow();

        let line_index = layout.find_line_index_by_position(position)?;
        if line_index + 1 < self.scroll() {
            // Return null for lines above the viewport.
            // TODO It also should return null for lines below the viewport,
            //      but the logic is not trivial.
            return None;
        }

        let line = layout.lines().get(line_index)?;
        let bounds = line.char_bounds(position)?;
        let bounds = self.layout_to_local_matrix() * bounds;

        // FP does not apply hscroll to char boundaries, so just revert it.
        // TODO Check if that's fixed in versions newer than 32.
        let bounds =
            Matrix::translate(Twips::from_pixels(self.0.hscroll.get()), Twips::ZERO) * bounds;
        Some(bounds)
    }

    fn execute_avm1_asfunction(
        self,
        context: &mut UpdateContext<'gc>,
        address: &WStr,
    ) -> Result<(), crate::avm1::Error<'gc>> {
        let Some(parent) = self.avm1_parent() else {
            return Ok(()); // Can't open links for something that isn't visible?
        };

        let mut activation = Avm1Activation::from_nothing(
            context,
            ActivationIdentifier::root("[EditText URL]"),
            parent,
        );
        // [NA]: Should all `from_nothings` be scoped to root? It definitely should here.
        activation.set_scope_to_display_object(parent);
        let this = parent.object1_or_undef();

        if let Some((name, args)) = address.split_once(b',') {
            let name = AvmString::new(activation.gc(), name);
            let args = AvmString::new(activation.gc(), args);
            let function = activation.get_variable(name)?;
            function.call_with_default_this(this, name, &mut activation, &[args.into()])?;
        } else {
            let name = AvmString::new(activation.gc(), address);
            let function = activation.get_variable(name)?;
            function.call_with_default_this(this, name, &mut activation, &[])?;
        }
        Ok(())
    }

    fn open_url(self, context: &mut UpdateContext<'gc>, url: &WStr, target: &WStr) {
        if let Some(address) = url.strip_prefix(WStr::from_units(b"asfunction:")) {
            if let Err(e) = self.execute_avm1_asfunction(context, address) {
                error!("Couldn't execute URL \"{url:?}\": {e:?}");
            }
        } else if let Some(address) = url.strip_prefix(WStr::from_units(b"event:")) {
            if let Some(object) = self.object2() {
                let mut activation = Avm2Activation::from_nothing(context);
                let text = AvmString::new(activation.gc(), address);
                let event = Avm2EventObject::text_event(&mut activation, "link", text, true, false);

                Avm2::dispatch_event(activation.context, event, object.into());
            }
        } else {
            context
                .navigator
                .navigate_to_url(&url.to_utf8_lossy(), &target.to_utf8_lossy(), None);
        }
    }

    fn is_link_at(self, point: Point<Twips>) -> bool {
        let Some(mut position) = self.global_to_local(point) else {
            return false;
        };
        position.x += Self::GUTTER + Twips::from_pixels(self.0.hscroll.get());
        position.y += Self::GUTTER + self.0.vertical_scroll_offset();

        self.0.layout.borrow().boxes_iter().any(|layout| {
            layout.is_link()
                && layout
                    .bounds()
                    .contains(Position::from((position.x, position.y)))
        })
    }

    fn handle_click(self, click_index: usize, position: usize) {
        if !self.is_selectable() {
            return;
        }

        let this_click = ClickEventData {
            position,
            click_index,
        };
        let selection_mode = this_click.selection_mode();
        self.0.last_click.set(Some(this_click));

        // Update selection
        let selection = self.calculate_selection_at(position, selection_mode);
        self.set_selection(Some(selection));
    }

    fn handle_drag(self, position: usize) {
        if !self.is_selectable() {
            return;
        }

        let Some((last_position, selection_mode)) = self
            .0
            .last_click
            .get()
            .map(|last_click| (last_click.position, last_click.selection_mode()))
        else {
            // No last click, so no drag
            return;
        };

        // We have to calculate selections at the first and the current position,
        // because the user may be selecting words or lines.
        let first_selection = self.calculate_selection_at(last_position, selection_mode);
        let current_selection = self.calculate_selection_at(position, selection_mode);
        let new_selection = TextSelection::span_across(first_selection, current_selection);
        self.set_selection(Some(new_selection));
    }

    pub fn set_avm2_class(self, mc: &Mutation<'gc>, class: Avm2ClassObject<'gc>) {
        unlock!(Gc::write(mc, self.0), EditTextData, class).set(Some(class));
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.raw_interactive())
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.shared.id
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.shared.swf.clone()
    }

    /// Construct objects placed on this frame.
    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && self.object2().is_none() {
            self.construct_as_avm2_object(context, self.into());
            self.on_construction_complete(context);
        }
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if !self.movie().is_action_script_3() {
            self.construct_as_avm1_object(context);
        }
    }

    fn object1(self) -> Option<Avm1Object<'gc>> {
        self.0.object.get().and_then(|o| o.as_avm1_object())
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.object.get().and_then(|o| o.as_avm2_object())
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        self.set_object(Some(to.into()), context.gc());
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        self.apply_autosize_bounds();

        self.0.bounds.get()
    }

    fn pixel_bounds(self) -> Rectangle<Twips> {
        // For pixel bounds we can't apply lazy autosize bounds.
        // It's a bit hacky, but it seems that pixelBounds are
        // an exception to the rule that lazy autosize bounds
        // are applied when reading anything related to bounds.
        let old = self.0.autosize_lazy_bounds.take();
        let bounds = self.world_bounds();
        self.0.autosize_lazy_bounds.set(old);
        bounds
    }

    // The returned position x and y of a text field is offset by the text bounds.
    fn x(self) -> Twips {
        self.apply_autosize_bounds();
        self.base().x() + self.bounds_x_offset()
    }

    fn set_x(self, x: Twips) {
        self.apply_autosize_bounds();
        let offset = self.bounds_x_offset();
        self.base().set_x(x - offset);
        self.invalidate_cached_bitmap();
    }

    fn y(self) -> Twips {
        self.apply_autosize_bounds();
        self.base().y() + self.bounds_y_offset()
    }

    fn set_y(self, y: Twips) {
        self.apply_autosize_bounds();
        let offset = self.bounds_y_offset();
        self.base().set_y(y - offset);
        self.invalidate_cached_bitmap();
    }

    fn width(self) -> f64 {
        self.apply_autosize_bounds();

        let bounds = self.0.bounds.get();
        (self.base().matrix() * bounds).width().to_pixels()
    }

    fn set_width(self, context: &mut UpdateContext<'gc>, value: f64) {
        self.apply_autosize_bounds();

        let bounds = &self.0.bounds;
        bounds.set(bounds.get().with_width(Twips::from_pixels(value)));
        self.base().set_transformed_by_script(true);
        self.relayout(context);
    }

    fn height(self) -> f64 {
        self.apply_autosize_bounds();

        let bounds = self.0.bounds.get();
        (self.base().matrix() * bounds).height().to_pixels()
    }

    fn set_height(self, context: &mut UpdateContext<'gc>, value: f64) {
        self.apply_autosize_bounds();

        let bounds = &self.0.bounds;
        bounds.set(bounds.get().with_height(Twips::from_pixels(value)));
        self.base().set_transformed_by_script(true);
        self.relayout(context);
    }

    fn set_matrix(self, matrix: Matrix) {
        self.base().set_matrix(matrix);
        self.invalidate_cached_bitmap();
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        self.apply_autosize_bounds();

        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        fn is_transform_positive_scale_only(context: &mut RenderContext) -> bool {
            let Matrix { a, b, c, d, .. } = context.transform_stack.transform().matrix;
            // Flash does allow small shear. The following value is higher than
            // expected due to the fact that the final calculated shear differs
            // between Flash and Ruffle, and using a precise value would hide
            // some objects that should otherwise be shown.
            const ALLOWED_SHEAR: f32 = 0.006;
            b.abs() < ALLOWED_SHEAR && c.abs() < ALLOWED_SHEAR && a > 0.0 && d > 0.0
        }

        // EditText is not rendered if device font is used
        // and if it's rotated, sheared, or reflected.
        if self.is_device_font() && !is_transform_positive_scale_only(context) {
            return;
        }

        if self
            .0
            .flags
            .get()
            .intersects(EditTextFlag::BORDER | EditTextFlag::HAS_BACKGROUND)
        {
            let background_color = Some(self.0.background_color.get())
                .filter(|_| self.0.flags.get().contains(EditTextFlag::HAS_BACKGROUND));
            let border_color = Some(self.0.border_color.get())
                .filter(|_| self.0.flags.get().contains(EditTextFlag::BORDER));

            if self.is_device_font() {
                self.draw_device_text_box(
                    context,
                    self.0.bounds.get(),
                    background_color,
                    border_color,
                );
            } else {
                self.draw_text_box(context, self.0.bounds.get(), background_color, border_color);
            }
        }

        context.commands.push_mask();

        let mask_bounds = self.0.bounds.get().grow_x(-Self::GUTTER);
        let mask = Matrix::create_box_from_rectangle(&mask_bounds);

        context.commands.draw_rect(
            Color::WHITE,
            context.transform_stack.transform().matrix * mask,
        );
        context.commands.activate_mask();

        context.transform_stack.push(&Transform {
            matrix: self.layout_to_local_matrix(),
            ..Default::default()
        });

        let mut render_state = Default::default();
        self.render_text(context, &mut render_state);

        self.render_debug_boxes(
            context,
            self.0.layout_debug_boxes_flags.get(),
            &self.0.layout.borrow(),
        );

        context.transform_stack.pop();

        context.commands.deactivate_mask();
        context.commands.draw_rect(
            Color::WHITE,
            context.transform_stack.transform().matrix * mask,
        );
        context.commands.pop_mask();

        if let Some(draw_caret_command) = render_state.draw_caret_command {
            context.commands.commands.push(draw_caret_command);
        }
    }

    fn allow_as_mask(self) -> bool {
        false
    }

    fn avm1_unload(self, context: &mut UpdateContext<'gc>) {
        self.drop_focus(context);

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc(), None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc(), None, true);
        }

        // Unbind any display objects bound to this text.
        if let Some(dobj) = self.0.bound_display_object.take() {
            Avm1TextFieldBinding::clear_binding(dobj, self, context.gc());
        }

        // Unregister any text fields that may be bound to *this* text field.
        Avm1TextFieldBinding::unregister_bindings(self.into(), context);

        if self.variable().is_some() {
            context
                .unbound_text_fields
                .retain(|&text_field| !DisplayObject::ptr_eq(text_field.into(), self.into()));
        }

        self.set_avm1_removed(true);
    }

    fn avm1_text_field_bindings(&self) -> Option<Ref<'_, [Avm1TextFieldBinding<'gc>]>> {
        self.0
            .object
            .get()
            .and_then(|o| o.as_avm1_object())
            .map(|_| Ref::map(self.0.avm1_text_field_bindings.borrow(), |r| &r[..]))
    }

    fn avm1_text_field_bindings_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, Vec<Avm1TextFieldBinding<'gc>>>> {
        self.0
            .object
            .get()
            .and_then(|o| o.as_avm1_object())
            .map(|_| {
                unlock!(
                    Gc::write(mc, self.0),
                    EditTextData,
                    avm1_text_field_bindings
                )
                .borrow_mut()
            })
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
        self,
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
                Matrix::create_box(width, 1.0, bounds.x_min - Twips::HALF_PX, bounds.y_min),
            );
            // Bottom
            context.commands.draw_line(
                border_color,
                Matrix::create_box(width, 1.0, bounds.x_min - Twips::HALF_PX, bounds.y_max),
            );
            // Left
            context.commands.draw_line(
                border_color,
                Matrix::create_box_with_rotation(
                    1.0,
                    height,
                    std::f32::consts::FRAC_PI_2,
                    bounds.x_min,
                    bounds.y_min - Twips::HALF_PX,
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
                    bounds.y_min - Twips::HALF_PX,
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
        self,
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

    fn ime_cursor_area(self) -> ImeCursorArea {
        // TODO We should be smarter here and return an area closer to the cursor.
        let bounds = self.world_bounds();
        ImeCursorArea {
            x: bounds.x_min.to_pixels(),
            y: bounds.y_min.to_pixels(),
            width: bounds.width().to_pixels(),
            height: bounds.height().to_pixels(),
        }
    }
}

impl<'gc> TInteractiveObject<'gc> for EditText<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        event: ClipEvent,
    ) -> ClipEventResult {
        match event {
            ClipEvent::Press { .. } | ClipEvent::MouseWheel { .. } | ClipEvent::MouseMove => {
                ClipEventResult::Handled
            }
            _ => ClipEventResult::NotHandled,
        }
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if let ClipEvent::MouseWheel { delta } = event {
            let scrolled = if self.is_mouse_wheel_enabled() {
                let new_scroll = self.scroll() as f64 - delta.lines();
                let scrolled = self.set_scroll(new_scroll);

                let mut activation = Avm1Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[On Scroller]"),
                    self.into(),
                );
                self.on_scroller(&mut activation);
                scrolled
            } else {
                false
            };

            if scrolled {
                return ClipEventResult::Handled;
            } else {
                return ClipEventResult::NotHandled;
            }
        }

        if let ClipEvent::Press { index } = event {
            // We can't hold self as any link may end up modifying this object, so pull the info out
            let mut link_to_open = None;

            if let Some(position) = self.screen_position_to_index(*context.mouse_position) {
                self.handle_click(index, position);

                if let Some((span_index, _)) = self
                    .0
                    .text_spans
                    .borrow()
                    .resolve_position_as_span(position)
                {
                    link_to_open = self
                        .0
                        .text_spans
                        .borrow()
                        .span(span_index)
                        .map(|s| (s.url.clone(), s.target.clone()));
                }
            } else {
                self.set_selection(Some(TextSelection::for_position(self.text_length())));
            }

            if let Some((url, target)) = link_to_open
                && !url.is_empty()
            {
                // TODO: This fires on mouse DOWN but it should be mouse UP...
                // but only if it went down in the same span.
                // Needs more advanced focus handling than we have at time of writing this comment.
                // TODO This also needs to fire only if the user clicked on the link,
                //   currently it fires when the cursor position resolves to one in the link.
                self.open_url(context, &url, &target);
            }

            return ClipEventResult::Handled;
        }

        if let ClipEvent::MouseMove = event {
            // If a mouse has moved and this EditTest is pressed, we need to update the selection.
            if InteractiveObject::option_ptr_eq(context.mouse_data.pressed, Some(self.into()))
                && let Some(position) = self.screen_position_to_index(*context.mouse_position)
            {
                self.handle_drag(position);
            }
        }

        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        self,
        context: &mut UpdateContext<'gc>,
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
            Some(self.into())
        } else {
            None
        }
    }

    fn mouse_pick_avm2(
        self,
        context: &mut UpdateContext<'gc>,
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
                Avm2MousePick::Hit(self.into())
            } else {
                Avm2MousePick::PropagateToParent
            }
        } else {
            Avm2MousePick::Miss
        }
    }

    fn mouse_cursor(self, context: &mut UpdateContext<'gc>) -> MouseCursor {
        if self.is_link_at(*context.mouse_position) {
            MouseCursor::Hand
        } else if self.is_selectable() {
            MouseCursor::IBeam
        } else {
            MouseCursor::Arrow
        }
    }

    fn on_focus_changed(
        self,
        context: &mut UpdateContext<'gc>,
        focused: bool,
        _other: Option<InteractiveObject<'gc>>,
    ) {
        if !focused {
            // Commit IME on focus lost.
            self.ensure_ime_committed(context);

            let is_avm1 = !self.movie().is_action_script_3();
            if is_avm1 {
                // Clear selection on focus lost in AVM1 only.
                self.set_selection(None);
            }
        }

        // Notify about IME
        context.send_notification(PlayerNotification::ImeNotification(if focused {
            ImeNotification::ImeReady {
                purpose: if self.is_password() {
                    ImePurpose::Password
                } else {
                    ImePurpose::Standard
                },
                cursor_area: self.ime_cursor_area(),
            }
        } else {
            ImeNotification::ImeNotReady
        }));
    }

    fn is_focusable_by_mouse(self, _context: &mut UpdateContext<'gc>) -> bool {
        self.movie().is_action_script_3() || self.is_editable() || self.is_selectable()
    }

    fn is_highlightable(self, _context: &mut UpdateContext<'gc>) -> bool {
        // TextField is incapable of rendering a highlight.
        false
    }

    fn is_tabbable(self, context: &mut UpdateContext<'gc>) -> bool {
        if !self.is_editable() {
            // Non-editable text fields are never tabbable.
            return false;
        }
        self.tab_enabled(context)
    }

    fn tab_enabled_default(self, _context: &mut UpdateContext<'gc>) -> bool {
        self.is_editable()
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    struct EditTextFlag: u16 {
        const FIRING_VARIABLE_BINDING = 1 << 0;
        const HAS_BACKGROUND = 1 << 1;
        const CONDENSE_WHITE = 1 << 13;
        const ALWAYS_SHOW_SELECTION = 1 << 14;

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

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct LayoutDebugBoxesFlag: u8 {
        const TEXT = 1 << 2;
        const LINE = 1 << 3;
        const BOX = 1 << 5;
        const CHAR = 1 << 7;
    }
}

/// Data shared between all instances of a text object.
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
struct EditTextShared {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    initial_text: Option<WString>,
}

#[derive(Clone, Copy, Debug)]
struct ClickEventData {
    /// The position in text resolved from click coordinates.
    position: usize,

    click_index: usize,
}

impl ClickEventData {
    /// Selection mode that results from this click index.
    #[inline]
    fn selection_mode(self) -> TextSelectionMode {
        TextSelectionMode::from_click_index(self.click_index)
    }
}

#[derive(Copy, Clone, Debug)]
enum TextSelectionMode {
    /// Specifies that text should be selected at char boundaries.
    ///
    /// Used when e.g. clicking or clicking and dragging.
    Character,

    /// Specifies that text should be selected at word boundaries.
    ///
    /// Used when e.g. double-clicking or double-clicking and dragging.
    Word,

    /// Specifies that text should be selected at line boundaries.
    ///
    /// Used when e.g. triple-clicking or triple-clicking and dragging.
    Line,
}

impl TextSelectionMode {
    fn from_click_index(click_index: usize) -> Self {
        match click_index {
            0 => Self::Character,
            1 => Self::Word,
            _ => Self::Line,
        }
    }
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

    /// Create a new selection spanning across the given selections.
    pub fn span_across(from: Self, to: Self) -> Self {
        let from_start = from.start();
        let from_end = from.end();
        let to_start = to.start();
        let to_end = to.end();
        if from_start < to_start && from_end < to_end {
            Self::for_range(from_start, to_end)
        } else if to_start < from_start && to_end < from_end {
            Self::for_range(from_end, to_start)
        } else {
            Self::for_range(from_start.min(to_start), from_end.max(to_end))
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

    fn intervals_contain(&self, character: char, intervals: &[(char, char)]) -> bool {
        for &interval in intervals {
            if self.interval_contains(character, interval) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn interval_contains(&self, character: char, interval: (char, char)) -> bool {
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

    pub fn filter_allowed(&self, text: &WStr) -> WString {
        let mut filtered = WString::with_capacity(text.len(), text.is_wide());
        for c in text.chars() {
            if let Some(c) = c.ok().and_then(|c| self.to_allowed(c)) {
                filtered.push_char(c);
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

#[derive(Clone, Copy, Collect, Debug, Default)]
#[collect(no_drop)]
enum EditTextStyleSheet<'gc> {
    #[default]
    None,
    Avm1(Avm1Object<'gc>),
    Avm2(Avm2StyleSheetObject<'gc>),
}

impl<'gc> EditTextStyleSheet<'gc> {
    fn is_some(self) -> bool {
        self.style_sheet().is_some()
    }

    fn is_none(self) -> bool {
        self.style_sheet().is_none()
    }

    fn style_sheet(self) -> Option<StyleSheet<'gc>> {
        match self {
            EditTextStyleSheet::None => None,
            EditTextStyleSheet::Avm1(object) => {
                if let Avm1NativeObject::StyleSheet(style_sheet_object) = object.native() {
                    Some(style_sheet_object.style_sheet())
                } else {
                    None
                }
            }
            EditTextStyleSheet::Avm2(style_sheet_object) => Some(style_sheet_object.style_sheet()),
        }
    }
}

#[derive(Clone, Debug)]
struct ImeData {
    ime_start: usize,
    ime_end: usize,
    text: String,
}

#[derive(Clone, Debug, Default)]
struct EditTextRenderState {
    /// Used for delaying rendering the caret, so that it's
    /// rendered outside of the text mask.
    draw_caret_command: Option<RenderCommand>,
}
