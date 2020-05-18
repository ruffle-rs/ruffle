//! `EditText` display object and support code.
use crate::avm1::globals::text_field::attach_virtual_properties;
use crate::avm1::{Avm1, Object, StageObject, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::font::{Font, Glyph};
use crate::html::{BoxBounds, FormatSpans, LayoutBox, Size, TextFormat};
use crate::library::Library;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use crate::xml::{XMLDocument, XMLNode};
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use std::sync::Arc;
use swf::Twips;

/// Boxed error type.
pub type Error = Box<dyn std::error::Error>;

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

    /// The calculated layout box.
    layout: Option<GcCell<'gc, LayoutBox<'gc>>>,

    // The AVM1 object handle
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

        if swf_tag.is_html {
            document
                .as_node()
                .replace_with_str(context.gc_context, &text)
                .unwrap();
            text_spans.lower_from_html(document);
        } else {
            text_spans.replace_text(0, text_spans.text().len(), &text, Some(&default_format));
        }

        let bounds: BoxBounds<Twips> = swf_tag.bounds.clone().into();
        let max_length = swf_tag
            .max_length
            .map(|ml| Twips::from_pixels(ml.into()))
            .unwrap_or_else(|| bounds.width());

        let layout =
            LayoutBox::lower_from_text_spans(&text_spans, context, swf_movie.clone(), max_length);

        EditText(GcCell::allocate(
            context.gc_context,
            EditTextData {
                base: Default::default(),
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
                object: None,
                layout,
            },
        ))
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
                x_max: Twips::from_pixels(x + height),
                y_min: Twips::from_pixels(y),
                y_max: Twips::from_pixels(y + width),
            },
            font_id: None,
            font_class_name: None,
            height: Some(Twips::from_pixels(height)),
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

        edit_text.text_spans.replace_text(0, len, &text, None);

        drop(edit_text);

        self.relayout(context);

        Ok(())
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
        self.relayout(context);
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

    /// Construct a base text transform for this `EditText`, to be used for
    /// evaluating fonts.
    ///
    /// The `text_transform` constitutes the base transform that all text is
    /// written into.
    ///
    /// The `text_transform` is separate from and relative to the base
    /// transform that this `EditText` automatically gets by virtue of being a
    /// `DisplayObject`.
    pub fn text_transform(self) -> Transform {
        let edit_text = self.0.read();
        let static_data = &edit_text.static_data;

        // TODO: Many of these properties should change be instance members instead
        // of static data, because they can be altered via ActionScript.
        let color = static_data
            .text
            .color
            .as_ref()
            .unwrap_or_else(|| &swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            });

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

    /// Relayout the `EditText`.
    ///
    /// This function operats exclusively with the text-span representation of
    /// the text, and no higher-level representation. Specifically, CSS should
    /// have already been calculated and applied to HTML trees lowered into the
    /// text-span representation.
    fn relayout(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut edit_text = self.0.write(context.gc_context);
        let movie = edit_text.static_data.swf.clone();
        let library = context.library.library_for_movie(movie.clone()).unwrap();

        if edit_text.is_multiline {
            //TODO: this should control if bounds are set during layout
        }

        let bounds: BoxBounds<Twips> = edit_text.static_data.text.bounds.clone().into();
        let max_length = edit_text
            .static_data
            .text
            .max_length
            .map(|ml| Twips::from_pixels(ml.into()))
            .unwrap_or_else(|| bounds.width());

        edit_text.layout =
            LayoutBox::lower_from_text_spans(&edit_text.text_spans, context, movie, max_length);
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, _context: &mut UpdateContext<'_, 'gc, '_>) -> (Twips, Twips) {
        let edit_text = self.0.read();
        let mut bounds: BoxBounds<Twips> = Default::default();
        let mut ptr = edit_text.layout;

        while let Some(layout_box) = ptr {
            bounds += layout_box.read().bounds();

            ptr = layout_box.read().next_sibling();
        }

        (bounds.width(), bounds.height())
    }

    /// Render a layout box, plus it's children.
    fn render_layout_box(
        &self,
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
        if let Some((start, end, _tf, font, font_size)) = lbox.read().text_node() {
            if let Some(chunk) = edit_text.text_spans.text().get(start..end) {
                font.evaluate(
                    &chunk[start..end],
                    self.text_transform(),
                    font_size,
                    |transform, glyph: &Glyph| {
                        // Render glyph.
                        context.transform_stack.push(transform);
                        context
                            .renderer
                            .render_shape(glyph.shape, context.transform_stack.transform());
                        context.transform_stack.pop();
                    },
                );
            }
        }

        context.transform_stack.pop();
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    impl_display_object!(base);

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
    }

    fn object(&self) -> Value<'gc> {
        self.0
            .read()
            .object
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().static_data.text.bounds.clone().into()
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&*self.transform());

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
