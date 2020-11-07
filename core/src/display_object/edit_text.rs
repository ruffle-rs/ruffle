//! `EditText` display object and support code.
use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::globals::text_field::attach_virtual_properties;
use crate::avm1::{Avm1, AvmString, Object, StageObject, TObject, Value};
use crate::backend::input::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::drawing::Drawing;
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult, KeyCode};
use crate::font::{round_down_to_pixel, Glyph};
use crate::html::{BoxBounds, FormatSpans, LayoutBox, LayoutContent, TextFormat};
use crate::prelude::*;
use crate::shape_utils::DrawCommand;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use crate::xml::XMLDocument;
use chrono::Utc;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use std::{cell::Ref, sync::Arc};
use swf::Twips;

/// Boxed error type.
pub type Error = Box<dyn std::error::Error>;

/// The kind of autosizing behavior an `EditText` should have, if any
#[derive(Copy, Clone, Debug, Collect)]
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
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct EditText<'gc>(GcCell<'gc, EditTextData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct EditTextData<'gc> {
    /// DisplayObject common properties.
    base: DisplayObjectBase<'gc>,

    /// Static data shared among all instances of this `EditText`.
    static_data: Gc<'gc, EditTextStatic>,

    /// The current HTML document displayed by this `EditText`.
    ///
    /// The HTML representation of this `EditText` is lowered into an
    /// appropriate set of format spans, which is used for actual rendering.
    /// The HTML is only retained if there is also a stylesheet already defined
    /// on the `EditText`, else it is discarded during the lowering process.
    document: XMLDocument<'gc>,

    /// The underlying text format spans of the `EditText`.
    ///
    /// This is generated from HTML (with optional CSS) or set directly, and
    /// can be directly manipulated by ActionScript. It can also be raised to
    /// an equivalent HTML representation, as long as no stylesheet is present.
    ///
    /// It is lowered further into layout boxes, which are used for actual
    /// rendering.
    text_spans: FormatSpans,

    /// If the text is in multi-line mode or single-line mode.
    is_multiline: bool,

    /// If the text can be selected by the user.
    is_selectable: bool,

    /// If the text can be edited by the user.
    is_editable: bool,

    /// If the text is word-wrapped.
    is_word_wrap: bool,

    /// If the text field should have a border.
    has_border: bool,

    /// If the text field is required to use device fonts only.
    is_device_font: bool,

    /// If the text field renders as HTML.
    is_html: bool,

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
    bounds: BoundingBox,

    /// The AVM1 object handle
    object: Option<Object<'gc>>,

    /// The variable path that this text field is bound to (AVM1 only).
    variable: Option<String>,

    /// The display object that the variable binding is bound to.
    bound_stage_object: Option<StageObject<'gc>>,

    /// Whether this text field is firing is variable binding (to prevent infinite loops).
    firing_variable_binding: bool,

    /// The selected portion of the text, or None if the text is not selected.
    selection: Option<TextSelection>,

    /// Whether or not this EditText has the current keyboard focus
    has_focus: bool,
}

impl<'gc> EditText<'gc> {
    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_movie: Arc<SwfMovie>,
        swf_tag: swf::EditText,
    ) -> Self {
        let is_multiline = swf_tag.is_multiline;
        let is_word_wrap = swf_tag.is_word_wrap;
        let is_selectable = swf_tag.is_selectable;
        let is_editable = !swf_tag.is_read_only;
        let is_html = swf_tag.is_html;
        let document = XMLDocument::new(context.gc_context);
        let text = swf_tag.initial_text.clone().unwrap_or_default();
        let default_format = TextFormat::from_swf_tag(swf_tag.clone(), swf_movie.clone(), context);

        let mut text_spans = FormatSpans::new();
        text_spans.set_default_format(default_format.clone());

        if is_html {
            let _ = document
                .as_node()
                .replace_with_str(context.gc_context, &text, false);
            text_spans.lower_from_html(document);
        } else {
            text_spans.replace_text(0, text_spans.text().len(), &text, Some(&default_format));
        }

        if !is_multiline {
            let filtered = text_spans.text().replace("\n", "");
            text_spans.replace_text(0, text_spans.text().len(), &filtered, Some(&default_format));
        }

        let bounds: BoundingBox = swf_tag.bounds.clone().into();

        let (layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &text_spans,
            context,
            swf_movie.clone(),
            bounds.width() - Twips::from_pixels(Self::INTERNAL_PADDING * 2.0),
            swf_tag.is_word_wrap,
            swf_tag.is_device_font,
        );

        let has_border = swf_tag.has_border;
        let is_device_font = swf_tag.is_device_font;

        let mut base = DisplayObjectBase::default();

        base.matrix_mut(context.gc_context).tx = bounds.x_min;
        base.matrix_mut(context.gc_context).ty = bounds.y_min;

        let variable = if !swf_tag.variable_name.is_empty() {
            Some(swf_tag.variable_name.clone())
        } else {
            None
        };

        let et = EditText(GcCell::allocate(
            context.gc_context,
            EditTextData {
                base,
                document,
                text_spans,
                static_data: gc_arena::Gc::allocate(
                    context.gc_context,
                    EditTextStatic {
                        swf: swf_movie,
                        text: swf_tag,
                    },
                ),
                is_multiline,
                is_selectable,
                is_editable,
                is_word_wrap,
                has_border,
                is_device_font,
                is_html,
                drawing: Drawing::new(),
                object: None,
                layout,
                intrinsic_bounds,
                bounds,
                autosize: AutoSizeMode::None,
                variable,
                bound_stage_object: None,
                firing_variable_binding: false,
                selection: None,
                has_focus: false,
            },
        ));

        et.redraw_border(context.gc_context);

        et
    }

    /// Create a new, dynamic `EditText`.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_movie: Arc<SwfMovie>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let swf_tag = swf::EditText {
            id: 0, //TODO: Dynamic text fields don't have a character ID?
            bounds: swf::Rectangle {
                x_min: Twips::from_pixels(0.0),
                x_max: Twips::from_pixels(width),
                y_min: Twips::from_pixels(0.0),
                y_max: Twips::from_pixels(height),
            },
            font_id: None,
            font_class_name: None,
            height: Some(Twips::from_pixels(12.0)),
            color: Some(swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0xFF,
            }),
            max_length: Some(width as u16),
            layout: Some(swf::TextLayout {
                align: swf::TextAlign::Left,
                left_margin: Twips::from_pixels(0.0),
                right_margin: Twips::from_pixels(0.0),
                indent: Twips::from_pixels(0.0),
                leading: Twips::from_pixels(0.0),
            }),
            variable_name: "".to_string(), //TODO: should be null
            initial_text: None,
            is_word_wrap: false,
            is_multiline: false,
            is_password: false,
            is_read_only: true,
            is_auto_size: false,
            is_selectable: true,
            has_border: false,
            was_static: false,
            is_html: false,
            is_device_font: false,
        };

        let text_field = Self::from_swf_tag(context, swf_movie, swf_tag);

        // Set position.
        let mut matrix = text_field.matrix_mut(context.gc_context);
        matrix.tx = Twips::from_pixels(x);
        matrix.ty = Twips::from_pixels(y);
        drop(matrix);

        text_field
    }

    pub fn text(self) -> String {
        self.0.read().text_spans.text().to_string()
    }

    pub fn set_text(
        self,
        text: String,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut edit_text = self.0.write(context.gc_context);
        let len = edit_text.text_spans.text().len();
        let tf = edit_text.text_spans.default_format().clone();

        edit_text.text_spans.replace_text(0, len, &text, Some(&tf));

        drop(edit_text);

        self.relayout(context);

        Ok(())
    }

    pub fn html_text(self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<String, Error> {
        if self.is_html() {
            let html_tree = self.html_tree(context).as_node();
            let html_string_result = html_tree.into_string(&mut |_node| true);

            if let Err(err) = &html_string_result {
                log::warn!(
                    "Serialization error when reading TextField.htmlText: {}",
                    err
                );
            }

            Ok(html_string_result.unwrap_or_else(|_| "".to_string()))
        } else {
            // Non-HTML text fields always return plain text.
            Ok(self.text())
        }
    }

    pub fn set_html_text(
        self,
        text: String,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if self.is_html() {
            let html_string = text.replace("<sbr>", "\n").replace("<br>", "\n");
            let document = XMLDocument::new(context.gc_context);

            if let Err(err) =
                document
                    .as_node()
                    .replace_with_str(context.gc_context, &html_string, false)
            {
                log::warn!("Parsing error when setting TextField.htmlText: {}", err);
            }

            self.set_html_tree(document, context);
        } else if let Err(err) = self.set_text(text, context) {
            log::error!("Error when setting TextField.htmlText: {}", err);
        }
        Ok(())
    }

    pub fn html_tree(self, context: &mut UpdateContext<'_, 'gc, '_>) -> XMLDocument<'gc> {
        self.0.read().text_spans.raise_to_html(context.gc_context)
    }

    /// Set the HTML tree for the given display object.
    ///
    /// The document is not rendered directly: instead, it is lowered to text
    /// spans which drive the actual layout process. User code is capable of
    /// altering text spans directly, thus the HTML tree will be discarded and
    /// regenerated.
    ///
    /// In stylesheet mode, the opposite is true: text spans are an
    /// intermediate, user-facing text span APIs don't work, and the document
    /// is retained.
    pub fn set_html_tree(self, doc: XMLDocument<'gc>, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut write = self.0.write(context.gc_context);

        write.document = doc;
        write.text_spans.lower_from_html(doc);

        drop(write);

        self.relayout(context);
    }

    pub fn text_length(self) -> usize {
        self.0.read().text_spans.text().len()
    }

    pub fn new_text_format(self) -> TextFormat {
        self.0.read().text_spans.default_format().clone()
    }

    pub fn set_new_text_format(self, tf: TextFormat, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0
            .write(context.gc_context)
            .text_spans
            .set_default_format(tf);
    }

    pub fn text_format(self, from: usize, to: usize) -> TextFormat {
        self.0.read().text_spans.get_text_format(from, to)
    }

    pub fn set_text_format(
        self,
        from: usize,
        to: usize,
        tf: TextFormat,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        self.0
            .write(context.gc_context)
            .text_spans
            .set_text_format(from, to, &tf);
        self.relayout(context);
    }

    pub fn is_editable(self) -> bool {
        self.0.read().is_editable
    }

    pub fn set_editable(self, is_editable: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_editable = is_editable;
    }

    pub fn is_multiline(self) -> bool {
        self.0.read().is_multiline
    }

    pub fn set_multiline(self, is_multiline: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_multiline = is_multiline;
        self.relayout(context);
    }

    pub fn is_selectable(self) -> bool {
        self.0.read().is_selectable
    }

    pub fn set_selectable(self, is_selectable: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_selectable = is_selectable;
    }

    pub fn is_word_wrap(self) -> bool {
        self.0.read().is_word_wrap
    }

    pub fn set_word_wrap(self, is_word_wrap: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_word_wrap = is_word_wrap;
        self.relayout(context);
    }

    pub fn autosize(self) -> AutoSizeMode {
        self.0.read().autosize
    }

    pub fn set_autosize(self, asm: AutoSizeMode, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).autosize = asm;
        self.relayout(context);
    }

    pub fn has_border(self) -> bool {
        self.0.read().has_border
    }

    pub fn set_has_border(self, context: MutationContext<'gc, '_>, has_border: bool) {
        self.0.write(context).has_border = has_border;
        self.redraw_border(context);
    }

    pub fn is_device_font(self) -> bool {
        self.0.read().is_device_font
    }

    pub fn set_is_device_font(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        is_device_font: bool,
    ) {
        self.0.write(context.gc_context).is_device_font = is_device_font;
        self.relayout(context);
    }

    pub fn is_html(self) -> bool {
        self.0.read().is_html
    }

    pub fn set_is_html(self, context: &mut UpdateContext<'_, 'gc, '_>, is_html: bool) {
        self.0.write(context.gc_context).is_html = is_html;
    }

    pub fn replace_text(
        self,
        from: usize,
        to: usize,
        text: &str,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
    pub fn text_transform(self, color: swf::Color, baseline_adjustment: Twips) -> Transform {
        let mut transform: Transform = Default::default();
        transform.color_transform.r_mult = f32::from(color.r) / 255.0;
        transform.color_transform.g_mult = f32::from(color.g) / 255.0;
        transform.color_transform.b_mult = f32::from(color.b) / 255.0;
        transform.color_transform.a_mult = f32::from(color.a) / 255.0;

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

        if let Some(layout) = &static_data.text.layout {
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

    pub fn set_variable(self, variable: Option<String>, activation: &mut Activation<'_, 'gc, '_>) {
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
            .text
            .initial_text
            .clone()
            .unwrap_or_default();
        let _ = self.set_text(text, &mut activation.context);

        self.0.write(activation.context.gc_context).variable = variable;
        self.try_bind_text_field_variable(activation, true);
    }

    /// Construct a base text transform for this `EditText`, to be used for
    /// evaluating fonts.
    ///
    /// The `text_transform` constitutes the base transform that all text is
    /// written into.

    /// Redraw the border of this `EditText`.
    fn redraw_border(self, context: MutationContext<'gc, '_>) {
        let mut write = self.0.write(context);

        write.drawing.clear();

        if write.has_border {
            let bounds = write.bounds.clone();

            write.drawing.set_line_style(Some(swf::LineStyle::new_v1(
                Twips::new(1),
                swf::Color::from_rgb(0, 0xFF),
            )));
            write
                .drawing
                .set_fill_style(Some(swf::FillStyle::Color(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                })));
            write.drawing.draw_command(DrawCommand::MoveTo {
                x: Twips::new(0),
                y: Twips::new(0),
            });
            write.drawing.draw_command(DrawCommand::LineTo {
                x: Twips::new(0),
                y: bounds.y_max - bounds.y_min,
            });
            write.drawing.draw_command(DrawCommand::LineTo {
                x: bounds.x_max - bounds.x_min,
                y: bounds.y_max - bounds.y_min,
            });
            write.drawing.draw_command(DrawCommand::LineTo {
                x: bounds.x_max - bounds.x_min,
                y: Twips::new(0),
            });
            write.drawing.draw_command(DrawCommand::LineTo {
                x: Twips::new(0),
                y: Twips::new(0),
            });
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
    fn relayout(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut edit_text = self.0.write(context.gc_context);
        let autosize = edit_text.autosize;
        let is_word_wrap = edit_text.is_word_wrap;
        let movie = edit_text.static_data.swf.clone();
        let width = edit_text.bounds.width() - Twips::from_pixels(Self::INTERNAL_PADDING * 2.0);

        let (new_layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &edit_text.text_spans,
            context,
            movie,
            width,
            is_word_wrap,
            edit_text.is_device_font,
        );

        edit_text.layout = new_layout;
        edit_text.intrinsic_bounds = intrinsic_bounds;

        match autosize {
            AutoSizeMode::None => {}
            AutoSizeMode::Left => {
                if !is_word_wrap {
                    let old_x = edit_text.bounds.x_min;
                    edit_text.bounds.set_x(old_x);
                    edit_text.base.set_x(old_x.to_pixels());
                    edit_text.bounds.set_width(intrinsic_bounds.width());
                }

                edit_text.bounds.set_height(intrinsic_bounds.height());
                edit_text.base.set_transformed_by_script(true);
            }
            AutoSizeMode::Center => {
                if !is_word_wrap {
                    let old_x = edit_text.bounds.x_min;
                    let new_x = (intrinsic_bounds.width() - old_x) / 2;
                    edit_text.bounds.set_x(new_x);
                    edit_text.base.set_x(new_x.to_pixels());
                    edit_text.bounds.set_width(intrinsic_bounds.width());
                }

                edit_text.bounds.set_height(intrinsic_bounds.height());
                edit_text.base.set_transformed_by_script(true);
            }
            AutoSizeMode::Right => {
                if !is_word_wrap {
                    let old_x = edit_text.bounds.x_min;
                    let new_x = intrinsic_bounds.width() - old_x;
                    edit_text.bounds.set_x(new_x);
                    edit_text.base.set_x(new_x.to_pixels());
                    edit_text.bounds.set_width(intrinsic_bounds.width());
                }

                edit_text.bounds.set_height(intrinsic_bounds.height());
                edit_text.base.set_transformed_by_script(true);
            }
        }
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, _context: &mut UpdateContext<'_, 'gc, '_>) -> (Twips, Twips) {
        let edit_text = self.0.read();

        (
            round_down_to_pixel(edit_text.intrinsic_bounds.width()),
            round_down_to_pixel(edit_text.intrinsic_bounds.height()),
        )
    }

    /// Render a layout box, plus its children.
    fn render_layout_box(self, context: &mut RenderContext<'_, 'gc>, lbox: &LayoutBox<'gc>) {
        let box_transform: Transform = lbox.bounds().origin().into();
        context.transform_stack.push(&box_transform);

        let edit_text = self.0.read();
        let selection = edit_text.selection;

        let caret = if let LayoutContent::Text { start, end, .. } = &lbox.content() {
            if let Some(selection) = selection {
                if selection.is_caret()
                    && edit_text.is_editable
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

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some((text, _tf, font, params, color)) =
            lbox.as_renderable_text(edit_text.text_spans.text())
        {
            let baseline_adjustmnet =
                font.get_baseline_for_height(params.height()) - params.height();
            font.evaluate(
                text,
                self.text_transform(color, baseline_adjustmnet),
                params,
                |pos, transform, glyph: &Glyph, advance, x| {
                    // If it's highlighted, override the color.
                    // TODO: We should draw a black background and change the color to white,
                    //  but for now let's just change it to be slightly different colour.
                    match selection {
                        Some(selection) if selection.contains(pos) => {
                            context.transform_stack.push(&Transform {
                                matrix: transform.matrix,
                                color_transform: transform.color_transform
                                    * ColorTransform {
                                        r_mult: 0.75,
                                        g_mult: 0.75,
                                        b_mult: 0.75,
                                        a_mult: 1.0,
                                        r_add: 0.0,
                                        g_add: 0.0,
                                        b_add: 1.0,
                                        a_add: 0.0,
                                    },
                            });
                        }
                        _ => {
                            context.transform_stack.push(&transform);
                        }
                    }

                    // Render glyph.
                    context
                        .renderer
                        .render_shape(glyph.shape_handle, context.transform_stack.transform());
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
                            context
                                .renderer
                                .draw_rect(Color::from_rgb(0x000000, 0xFF), &caret);
                        } else if pos == length - 1 && caret_pos == length {
                            let caret = context.transform_stack.transform().matrix
                                * Matrix::create_box(
                                    1.0,
                                    params.height().to_pixels() as f32,
                                    0.0,
                                    x + advance,
                                    Twips::from_pixels(2.0),
                                );
                            context
                                .renderer
                                .draw_rect(Color::from_rgb(0x000000, 0xFF), &caret);
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
        activation: &mut Activation<'_, 'gc, '_>,
        set_initial_value: bool,
    ) -> bool {
        let mut bound = false;
        if let Some(var_path) = self.variable() {
            // Any previous binding should have been cleared.
            debug_assert!(self.0.read().bound_stage_object.is_none());

            // Avoid double-borrows by copying the string.
            // TODO: Can we avoid this somehow? Maybe when we have a better string type.
            let variable = (*var_path).to_string();
            drop(var_path);

            let parent = self.parent().unwrap();

            activation.run_with_child_frame_for_display_object(
                "[Text Field Binding]",
                parent,
                activation.context.swf.header().version,
                |activation| {
                    if let Ok(Some((object, property))) =
                        activation.resolve_variable_path(parent, &variable)
                    {
                        // If this text field was just created, we immediately propagate the text to the variable (or vice versa).
                        if set_initial_value {
                            // If the property exists on the object, we overwrite the text with the property's value.
                            if object.has_property(activation, property) {
                                let value = object.get(property, activation).unwrap();
                                let _ = self.set_text(
                                    value
                                        .coerce_to_string(activation)
                                        .unwrap_or_default()
                                        .to_string(),
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
        }

        bound
    }

    /// Unsets a bound display object from this text field.
    /// Does not change the unbound text field list.
    /// Caller is responsible for adding this text field to the unbound list, if necessary.
    pub fn clear_bound_stage_object(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).bound_stage_object = None;
    }

    /// Propagates a text change to the bound display object.
    ///
    pub fn propagate_text_binding(self, activation: &mut Activation<'_, 'gc, '_>) {
        if !self.0.read().firing_variable_binding {
            self.0
                .write(activation.context.gc_context)
                .firing_variable_binding = true;
            if let Some(variable) = self.variable() {
                // Avoid double-borrows by copying the string.
                // TODO: Can we avoid this somehow? Maybe when we have a better string type.
                let variable_path = variable.to_string();
                drop(variable);

                if let Ok(Some((object, property))) =
                    activation.resolve_variable_path(self.parent().unwrap(), &variable_path)
                {
                    let text = if self.0.read().is_html {
                        let html_tree = self.html_tree(&mut activation.context).as_node();
                        let html_string_result = html_tree.into_string(&mut |_node| true);
                        html_string_result.unwrap_or_default()
                    } else {
                        self.text()
                    };

                    // Note that this can call virtual setters, even though the opposite direction won't work
                    // (virtual property changes do not affect the text field)
                    activation.run_with_child_frame_for_display_object(
                        "[Propagate Text Binding]",
                        self.parent().unwrap(),
                        activation.context.swf.header().version,
                        |activation| {
                            let _ = object.set(
                                property,
                                AvmString::new(activation.context.gc_context, text).into(),
                                activation,
                            );
                        },
                    );
                }
            }
            self.0
                .write(activation.context.gc_context)
                .firing_variable_binding = false;
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

    pub fn screen_position_to_index(self, position: (Twips, Twips)) -> Option<usize> {
        let text = self.0.read();
        let position = self.global_to_local(position);
        let position = (
            position.0 + Twips::from_pixels(Self::INTERNAL_PADDING),
            position.1 + Twips::from_pixels(Self::INTERNAL_PADDING),
        );

        for layout_box in text.layout.iter() {
            let transform: Transform = layout_box.bounds().origin().into();
            let mut matrix = transform.matrix;
            matrix.invert();
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
                        if local_position.0 >= x
                            && local_position.0 <= x + advance
                            && local_position.1 >= Twips::zero()
                            && local_position.1 <= params.height()
                        {
                            if local_position.0 >= x + (advance / 2) {
                                result = Some(pos + 1);
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

    pub fn text_input(self, character: char, context: &mut UpdateContext<'_, 'gc, '_>) {
        if !self.0.read().is_editable {
            return;
        }

        if let Some(selection) = self.selection() {
            let mut changed = false;
            match character as u8 {
                8 | 127 if !selection.is_caret() => {
                    // Backspace or delete with multiple characters selected
                    self.replace_text(selection.start(), selection.end(), "", context);
                    self.set_selection(
                        Some(TextSelection::for_position(selection.start())),
                        context.gc_context,
                    );
                    changed = true;
                }
                8 => {
                    // Backspace with caret
                    if selection.start() > 0 {
                        // Delete previous character
                        self.replace_text(selection.start() - 1, selection.start(), "", context);
                        self.set_selection(
                            Some(TextSelection::for_position(selection.start() - 1)),
                            context.gc_context,
                        );
                        changed = true;
                    }
                }
                127 => {
                    // Delete with caret
                    if selection.end() < self.text_length() {
                        // Delete next character
                        self.replace_text(selection.start(), selection.start() + 1, "", context);
                        // No need to change selection
                        changed = true;
                    }
                }
                32..=126 => {
                    // ASCII
                    // TODO: Make this actually good and not basic ASCII :)
                    self.replace_text(
                        selection.start(),
                        selection.end(),
                        &character.to_string(),
                        context,
                    );
                    self.set_selection(
                        Some(TextSelection::for_position(selection.start() + 1)),
                        context.gc_context,
                    );
                    changed = true;
                }
                _ => {}
            }

            if changed {
                let globals = context.avm1.global_object_cell();
                let swf_version = context.swf.header().version;
                let mut activation = Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Propagate Text Binding]"),
                    swf_version,
                    globals,
                    self.into(),
                );
                self.propagate_text_binding(&mut activation);
            }
        }
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    impl_display_object_sansbounds!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.text.id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().static_data.swf.clone())
    }

    fn run_frame(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        Some(*self)
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        _init_object: Option<Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        let mut text = self.0.write(context.gc_context);
        if text.object.is_none() {
            let object = StageObject::for_display_object(
                context.gc_context,
                display_object,
                Some(context.system_prototypes.text_field),
            )
            .into();

            attach_virtual_properties(
                context.gc_context,
                object,
                context.system_prototypes.function,
            );

            text.object = Some(object);
        }

        text.document = text
            .document
            .as_node()
            .duplicate(context.gc_context, true)
            .document();

        let mut new_layout = Vec::new();
        for layout_box in text.layout.iter() {
            new_layout.push(layout_box.duplicate(context.gc_context));
        }
        drop(text);

        // If this text field has a variable set, initialize text field binding.
        Avm1::run_with_stack_frame_for_display_object(
            (*self).into(),
            context.swf.version(),
            context,
            |activation| {
                if !self.try_bind_text_field_variable(activation, true) {
                    activation.context.unbound_text_fields.push(*self);
                }
                // People can bind to properties of TextFields the same as other display objects.
                self.bind_text_field_variables(activation);
            },
        );

        if run_frame {
            self.run_frame(context);
        }
    }

    fn object(&self) -> Value<'gc> {
        self.0
            .read()
            .object
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().bounds.clone()
    }

    // The returned position x and y of a text field is offset by the text bounds.
    fn x(&self) -> f64 {
        let edit_text = self.0.read();
        let offset = edit_text.bounds.x_min;
        (edit_text.base.transform.matrix.tx + offset).to_pixels()
    }

    fn set_x(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.x_min;
        edit_text.base.transform.matrix.tx = Twips::from_pixels(value) - offset;
        edit_text.base.set_transformed_by_script(true);
        drop(edit_text);
        self.redraw_border(gc_context);
    }

    fn y(&self) -> f64 {
        let edit_text = self.0.read();
        let offset = edit_text.bounds.y_min;
        (edit_text.base.transform.matrix.ty + offset).to_pixels()
    }

    fn set_y(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut edit_text = self.0.write(gc_context);
        let offset = edit_text.bounds.y_min;
        edit_text.base.transform.matrix.ty = Twips::from_pixels(value) - offset;
        edit_text.base.set_transformed_by_script(true);
        drop(edit_text);
        self.redraw_border(gc_context);
    }

    fn width(&self) -> f64 {
        self.0.read().bounds.width().to_pixels()
    }

    fn set_width(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_width(Twips::from_pixels(value));
        write.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn height(&self) -> f64 {
        self.0.read().bounds.height().to_pixels()
    }

    fn set_height(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_height(Twips::from_pixels(value));
        write.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn set_matrix(&self, context: MutationContext<'gc, '_>, matrix: &Matrix) {
        self.0.write(context).base.set_matrix(context, matrix);
        self.redraw_border(context);
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        if !self.world_bounds().intersects(&context.view_bounds) {
            // Off-screen; culled
            return;
        }

        let transform = self.transform().clone();
        context.transform_stack.push(&transform);

        let edit_text = self.0.read();
        context.transform_stack.push(&Transform {
            matrix: Matrix {
                tx: edit_text.bounds.x_min,
                ty: edit_text.bounds.y_min,
                ..Default::default()
            },
            ..Default::default()
        });

        edit_text.drawing.render(context);

        context.renderer.push_mask();
        let mask = Matrix::create_box(
            edit_text.bounds.width().to_pixels() as f32,
            edit_text.bounds.height().to_pixels() as f32,
            0.0,
            Twips::zero(),
            Twips::zero(),
        );
        context.renderer.draw_rect(
            Color::from_rgb(0, 0xff),
            &(context.transform_stack.transform().matrix * mask),
        );
        context.renderer.activate_mask();

        // TODO: Where does this come from? How is this different than INTERNAL_PADDING? Does this apply to y as well?
        // If this is actually right, offset the border in `redraw_border` instead of doing an extra push.
        context.transform_stack.push(&Transform {
            matrix: Matrix {
                tx: Twips::from_pixels(Self::INTERNAL_PADDING),
                ty: Twips::from_pixels(Self::INTERNAL_PADDING),
                ..Default::default()
            },
            ..Default::default()
        });

        if edit_text.layout.is_empty() && edit_text.is_editable {
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
                    context
                        .renderer
                        .draw_rect(Color::from_rgb(0x000000, 0xFF), &caret);
                }
            }
        } else {
            for layout_box in edit_text.layout.iter() {
                self.render_layout_box(context, layout_box);
            }
        }

        context.renderer.deactivate_mask();
        context.renderer.draw_rect(
            Color::from_rgb(0, 0xff),
            &(context.transform_stack.transform().matrix * mask),
        );
        context.renderer.pop_mask();

        context.transform_stack.pop();
        context.transform_stack.pop();
        context.transform_stack.pop();
    }

    fn allow_as_mask(&self) -> bool {
        false
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let had_focus = self.0.read().has_focus;
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }

        // Unbind any display objects bound to this text.
        if let Some(stage_object) = self.0.write(context.gc_context).bound_stage_object.take() {
            stage_object.clear_text_field_binding(context.gc_context, *self);
        }

        // Unregister any text fields that may be bound to *this* text field.
        if let Value::Object(object) = self.object() {
            if let Some(stage_object) = object.as_stage_object() {
                stage_object.unregister_text_field_bindings(context);
            }
        }
        if self.variable().is_some() {
            context
                .unbound_text_fields
                .retain(|&text_field| !DisplayObject::ptr_eq(text_field.into(), (*self).into()));
        }

        self.set_removed(context.gc_context, true);
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        self_node: DisplayObject<'gc>,
        point: (Twips, Twips),
    ) -> Option<DisplayObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() && self.is_selectable() && self.hit_test_shape(context, point) {
            Some(self_node)
        } else {
            None
        }
    }

    fn mouse_cursor(&self) -> MouseCursor {
        MouseCursor::IBeam
    }

    fn on_focus_changed(&self, context: MutationContext<'gc, '_>, focused: bool) {
        let mut text = self.0.write(context);
        text.has_focus = focused;
        if !focused {
            text.selection = None;
        }
    }

    fn is_focusable(&self) -> bool {
        // Even if this isn't selectable or editable, a script can focus on it manually
        true
    }

    fn handle_clip_event(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        match event {
            ClipEvent::Press => {
                let tracker = context.focus_tracker;
                tracker.set(Some((*self).into()), context);
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
            ClipEvent::KeyPress { key_code } => {
                let mut text = self.0.write(context.gc_context);
                let selection = text.selection;
                if let Some(mut selection) = selection {
                    let length = text.text_spans.text().len();
                    match key_code {
                        ButtonKeyCode::Left if selection.to > 0 => {
                            if context.input.is_key_down(KeyCode::Shift) {
                                selection.to -= 1;
                            } else {
                                selection.to -= 1;
                                selection.from = selection.to;
                            }
                        }
                        ButtonKeyCode::Right if selection.to < length => {
                            if context.input.is_key_down(KeyCode::Shift) {
                                selection.to += 1;
                            } else {
                                selection.to += 1;
                                selection.from = selection.to;
                            }
                        }
                        _ => {}
                    }
                    selection.clamp(length);
                    text.selection = Some(selection);
                    ClipEventResult::Handled
                } else {
                    ClipEventResult::NotHandled
                }
            }
            _ => ClipEventResult::NotHandled,
        }
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EditTextStatic {
    swf: Arc<SwfMovie>,
    text: swf::EditText,
}

unsafe impl<'gc> gc_arena::Collect for EditTextStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextSelection {
    from: usize,
    to: usize,
}

unsafe impl Collect for TextSelection {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
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

    /// The "end" part of the range is the smallest (closest to 0) part of this selection range.
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
