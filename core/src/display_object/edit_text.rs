//! `EditText` display object and support code.
use crate::avm1::globals::text_field::attach_virtual_properties;
use crate::avm1::{Avm1, Object, StageObject, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::drawing::Drawing;
use crate::font::{round_down_to_pixel, Glyph};
use crate::html::{BoxBounds, FormatSpans, LayoutBox, TextFormat};
use crate::prelude::*;
use crate::shape_utils::DrawCommand;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use crate::xml::XMLDocument;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use std::sync::Arc;
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

    /// If the text is word-wrapped.
    is_word_wrap: bool,

    /// If the text field should have a border.
    has_border: bool,

    /// The current border drawing.
    drawing: Drawing,

    /// Whether or not the width of the field should change in response to text
    /// changes, and in what direction should added or removed width should
    /// apply.
    autosize: AutoSizeMode,

    /// The calculated layout box.
    layout: Option<GcCell<'gc, LayoutBox<'gc>>>,

    /// The intrinsic bounds of the laid-out text.
    intrinsic_bounds: BoxBounds<Twips>,

    /// The current intrinsic bounds of the text field.
    bounds: BoundingBox,

    /// The AVM1 object handle
    object: Option<Object<'gc>>,
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
        let document = XMLDocument::new(context.gc_context);
        let text = swf_tag.initial_text.clone().unwrap_or_default();
        let default_format = TextFormat::from_swf_tag(swf_tag.clone(), swf_movie.clone(), context);

        let mut text_spans = FormatSpans::new();
        text_spans.set_default_format(default_format.clone());

        if swf_tag.is_html {
            document
                .as_node()
                .replace_with_str(context.gc_context, &text, false)
                .unwrap();
            text_spans.lower_from_html(document);
        } else {
            text_spans.replace_text(0, text_spans.text().len(), &text, Some(&default_format));
        }

        let bounds: BoundingBox = swf_tag.bounds.clone().into();

        let (layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &text_spans,
            context,
            swf_movie.clone(),
            bounds.width(),
            swf_tag.is_word_wrap,
            swf_tag.is_device_font,
        );

        let has_border = swf_tag.has_border;

        let mut base = DisplayObjectBase::default();

        base.matrix_mut(context.gc_context).tx = bounds.x_min;
        base.matrix_mut(context.gc_context).ty = bounds.y_min;

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
                is_word_wrap,
                has_border,
                drawing: Drawing::new(),
                object: None,
                layout,
                intrinsic_bounds,
                bounds,
                autosize: AutoSizeMode::None,
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
                x_min: Twips::from_pixels(x),
                x_max: Twips::from_pixels(x + width),
                y_min: Twips::from_pixels(y),
                y_max: Twips::from_pixels(y + height),
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

        Self::from_swf_tag(context, swf_movie, swf_tag)
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
    pub fn set_html_tree(
        &mut self,
        doc: XMLDocument<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
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

    pub fn is_multiline(self) -> bool {
        self.0.read().is_multiline
    }

    pub fn set_multiline(self, is_multiline: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_multiline = is_multiline;
        self.relayout(context);
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
    pub fn text_transform(self, color: swf::Color) -> Transform {
        let mut transform: Transform = Default::default();
        transform.color_transform.r_mult = f32::from(color.r) / 255.0;
        transform.color_transform.g_mult = f32::from(color.g) / 255.0;
        transform.color_transform.b_mult = f32::from(color.b) / 255.0;
        transform.color_transform.a_mult = f32::from(color.a) / 255.0;

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

    /// Internal padding between the bounds of the EditText and the maximum
    /// width of text inside the layout.
    const INTERNAL_PADDING: f64 = 7.0;

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
        let width = edit_text.bounds.width() - Twips::from_pixels(Self::INTERNAL_PADDING);

        let (new_layout, intrinsic_bounds) = LayoutBox::lower_from_text_spans(
            &edit_text.text_spans,
            context,
            movie,
            width,
            is_word_wrap,
            edit_text.static_data.text.is_device_font,
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

    /// Render a layout box, plus it's children.
    fn render_layout_box(
        self,
        context: &mut RenderContext<'_, 'gc>,
        lbox: GcCell<'gc, LayoutBox<'gc>>,
    ) {
        let box_transform: Transform = lbox.read().bounds().origin().into();
        context.transform_stack.push(&box_transform);

        let edit_text = self.0.read();

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some((text, _tf, font, params, color)) =
            lbox.read().as_renderable_text(edit_text.text_spans.text())
        {
            font.evaluate(
                text,
                self.text_transform(color),
                params,
                |transform, glyph: &Glyph, _advance| {
                    // Render glyph.
                    context.transform_stack.push(transform);
                    context
                        .renderer
                        .render_shape(glyph.shape, context.transform_stack.transform());
                    context.transform_stack.pop();
                },
            );
        }

        if let Some(drawing) = lbox.read().as_renderable_drawing() {
            drawing.render(context);
        }

        context.transform_stack.pop();
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

    fn run_frame(&mut self, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext) {
        // Noop
    }

    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        Some(*self)
    }

    fn post_instantiation(
        &mut self,
        _avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        _init_object: Option<Object<'gc>>,
        _instantiated_from_avm: bool,
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

            attach_virtual_properties(context.gc_context, object);

            text.object = Some(object);
        }

        text.document = text
            .document
            .as_node()
            .duplicate(context.gc_context, true)
            .document();

        text.layout = text.layout.map(|l| l.read().duplicate(context.gc_context));
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

    fn x(&self) -> f64 {
        self.0.read().bounds.x_min.to_pixels()
    }

    fn set_x(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_x(Twips::from_pixels(value));
        write.base.set_x(value);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn y(&self) -> f64 {
        self.0.read().bounds.y_min.to_pixels()
    }

    fn set_y(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_y(Twips::from_pixels(value));
        write.base.set_y(value);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn width(&self) -> f64 {
        self.0.read().bounds.width().to_pixels()
    }

    fn set_width(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_width(Twips::from_pixels(value));
        write.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn height(&self) -> f64 {
        self.0.read().bounds.height().to_pixels()
    }

    fn set_height(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let mut write = self.0.write(gc_context);

        write.bounds.set_height(Twips::from_pixels(value));
        write.base.set_transformed_by_script(true);

        drop(write);
        self.redraw_border(gc_context);
    }

    fn set_matrix(&mut self, context: MutationContext<'gc, '_>, matrix: &Matrix) {
        let mut write = self.0.write(context);

        let new_width = write.bounds.width().to_pixels() * matrix.a as f64;
        let new_height = write.bounds.height().to_pixels() * matrix.d as f64;

        write.bounds.set_width(Twips::from_pixels(new_width));
        write.bounds.set_height(Twips::from_pixels(new_height));

        let new_x = write.bounds.x_min + matrix.tx;
        let new_y = write.bounds.y_min + matrix.ty;

        write.bounds.set_x(new_x);
        write.bounds.set_y(new_y);

        write.base.set_matrix(context, matrix);

        drop(write);
        self.redraw_border(context);
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        let transform = self.transform().clone();
        context.transform_stack.push(&transform);

        self.0.read().drawing.render(context);

        let mut ptr = self.0.read().layout;

        while let Some(lbox) = ptr {
            self.render_layout_box(context, lbox);

            ptr = lbox.read().next_sibling();
        }

        context.transform_stack.pop();
    }

    fn allow_as_mask(&self) -> bool {
        false
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
