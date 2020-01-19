//! `EditText` display object and support code.
use crate::avm1::globals::text_field::attach_virtual_properties;
use crate::avm1::{Avm1, Object, ScriptObject, StageObject, TObject, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::font::Glyph;
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, Gc, GcCell, MutationContext};

type Error = Box<dyn std::error::Error>;

/// A set of text formatting options to be applied to some part, or the whole
/// of, a given text field.
///
/// Any property set to `None` is treated as undefined, which has different
/// meanings based on the context by which the `TextFormat` is used. For
/// example, when getting the format of a particular region of text, `None`
/// means that multiple regions of text apply. When setting the format of a
/// particular region of text, `None` means that the existing setting for that
/// property will be retained.
#[derive(Clone, Debug)]
pub struct TextFormat {
    font: Option<String>,
    size: Option<f64>,
    color: Option<swf::Color>,
    align: Option<swf::TextAlign>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    left_margin: Option<f64>,
    right_margin: Option<f64>,
    indent: Option<f64>,
    block_indent: Option<f64>,
    kerning: Option<bool>,
    leading: Option<f64>,
    letter_spacing: Option<f64>,
    tab_stops: Option<Vec<f64>>,
    bullet: Option<bool>,
    url: Option<String>,
    target: Option<String>,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            font: None,
            size: None,
            color: None,
            align: None,
            bold: None,
            italic: None,
            underline: None,
            left_margin: None,
            right_margin: None,
            indent: None,
            block_indent: None,
            kerning: None,
            leading: None,
            letter_spacing: None,
            tab_stops: None,
            bullet: None,
            url: None,
            target: None,
        }
    }
}

fn getstr_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<String>, Error> {
    Ok(match object.get(name, avm1, uc)?.resolve(avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.coerce_to_string(avm1, uc)?),
    })
}

fn getfloat_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<f64>, Error> {
    Ok(match object.get(name, avm1, uc)?.resolve(avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.as_number(avm1, uc)?),
    })
}

fn getbool_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<bool>, Error> {
    Ok(match object.get(name, avm1, uc)?.resolve(avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.as_bool(avm1.current_swf_version())),
    })
}

impl TextFormat {
    /// Construct a `TextFormat` from an object that is
    pub fn from_avm1_object<'gc>(
        object1: Object<'gc>,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        Ok(Self {
            font: getstr_from_avm1_object(object1, "font", avm1, uc)?,
            size: getfloat_from_avm1_object(object1, "size", avm1, uc)?,
            color: getfloat_from_avm1_object(object1, "color", avm1, uc)?.map(|v| swf::Color {
                r: ((v as u32 & 0xFF_0000) >> 16) as u8,
                g: ((v as u32 & 0x00_FF00) >> 8) as u8,
                b: (v as u32 & 0x00_00FF) as u8,
                a: 0xFF,
            }),
            align: getstr_from_avm1_object(object1, "align", avm1, uc)?.and_then(|v| {
                match v.to_lowercase().as_str() {
                    "left" => Some(swf::TextAlign::Left),
                    "center" => Some(swf::TextAlign::Center),
                    "right" => Some(swf::TextAlign::Right),
                    "justify" => Some(swf::TextAlign::Justify),
                    _ => None,
                }
            }),
            bold: getbool_from_avm1_object(object1, "bold", avm1, uc)?,
            italic: getbool_from_avm1_object(object1, "italic", avm1, uc)?,
            underline: getbool_from_avm1_object(object1, "underline", avm1, uc)?,
            left_margin: getfloat_from_avm1_object(object1, "leftMargin", avm1, uc)?,
            right_margin: getfloat_from_avm1_object(object1, "rightMargin", avm1, uc)?,
            indent: getfloat_from_avm1_object(object1, "indent", avm1, uc)?,
            block_indent: getfloat_from_avm1_object(object1, "blockIndent", avm1, uc)?,
            kerning: getbool_from_avm1_object(object1, "kerning", avm1, uc)?,
            leading: getfloat_from_avm1_object(object1, "leading", avm1, uc)?,
            letter_spacing: getfloat_from_avm1_object(object1, "letterSpacing", avm1, uc)?,
            tab_stops: None,
            bullet: getbool_from_avm1_object(object1, "bullet", avm1, uc)?,
            url: getstr_from_avm1_object(object1, "url", avm1, uc)?,
            target: getstr_from_avm1_object(object1, "target", avm1, uc)?,
        })
    }

    /// Construct a `TextFormat` AVM1 object from this text format object.
    pub fn as_avm1_object<'gc>(
        &self,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let object = ScriptObject::object(uc.gc_context, Some(avm1.prototypes().text_format));

        object.set(
            "font",
            self.font.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "size",
            self.size.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "color",
            self.color
                .clone()
                .map(|v| ((v.r << 16) + (v.g << 8) + v.b).into())
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "align",
            self.align
                .map(|v| {
                    match v {
                        swf::TextAlign::Left => "left",
                        swf::TextAlign::Center => "center",
                        swf::TextAlign::Right => "right",
                        swf::TextAlign::Justify => "justify",
                    }
                    .into()
                })
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bold",
            self.bold.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "italic",
            self.italic.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "underline",
            self.underline.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leftMargin",
            self.left_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "rightMargin",
            self.right_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "indent",
            self.indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "blockIndent",
            self.block_indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "kerning",
            self.kerning.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leading",
            self.leading.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "letterSpacing",
            self.letter_spacing.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bullet",
            self.bullet.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "url",
            self.url.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "target",
            self.target.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;

        Ok(object.into())
    }
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

#[derive(Clone, Debug)]
pub struct EditTextData<'gc> {
    /// DisplayObject common properties.
    base: DisplayObjectBase<'gc>,

    /// Static data shared among all instances of this `EditText`.
    static_data: Gc<'gc, EditTextStatic>,

    /// The current text displayed by this text field.
    text: String,

    /// The text formatting for newly inserted text spans.
    new_format: TextFormat,

    /// If the text is in multi-line mode or single-line mode.
    is_multiline: bool,

    // The AVM1 object handle
    object: Option<Object<'gc>>,
}

impl<'gc> EditText<'gc> {
    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(context: &mut UpdateContext<'_, 'gc, '_>, swf_tag: swf::EditText) -> Self {
        let is_multiline = swf_tag.is_multiline;

        EditText(GcCell::allocate(
            context.gc_context,
            EditTextData {
                base: Default::default(),
                text: swf_tag.initial_text.clone().unwrap_or_default(),
                new_format: TextFormat::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, EditTextStatic(swf_tag)),
                is_multiline,
                object: None,
            },
        ))
    }

    /// Create a new, dynamic `EditText`.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
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
            height: Some(height as u16),
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

        Self::from_swf_tag(context, swf_tag)
    }

    // TODO: This needs to strip away HTML
    pub fn text(self) -> String {
        self.0.read().text.to_owned()
    }

    pub fn set_text(self, text: String, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).text = text;
    }

    pub fn new_text_format(self) -> TextFormat {
        self.0.read().new_format.clone()
    }

    pub fn set_new_text_format(self, tf: TextFormat, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).new_format = tf;
    }

    pub fn is_multiline(self) -> bool {
        self.0.read().is_multiline
    }

    pub fn set_multiline(self, is_multiline: bool, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).is_multiline = is_multiline;
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
        let static_data = &edit_text.static_data.0;

        // TODO: Many of these properties should change be instance members instead
        // of static data, because they can be altered via ActionScript.
        let color = static_data.color.as_ref().unwrap_or_else(|| &swf::Color {
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

        if let Some(layout) = &static_data.layout {
            transform.matrix.ty -= layout.leading.get() as f32;
        }

        transform
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, context: &mut UpdateContext<'_, 'gc, '_>) -> (f32, f32) {
        let text_transform = self.text_transform();

        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);

        let mut size = (0.0, 0.0);

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some(font) = context
            .library
            .get_font(font_id)
            .filter(|font| font.has_glyphs())
            .or_else(|| context.library.device_font())
        {
            let height = static_data
                .height
                .map(|v| v as f32)
                .unwrap_or_else(|| font.scale());

            font.evaluate(
                &edit_text.text,
                text_transform,
                height,
                edit_text.static_data.0.is_html,
                |transform, glyph: &Glyph| {
                    //This is actually terrible matrix math.
                    let tx = transform.matrix.tx * transform.matrix.a;
                    let ty = transform.matrix.ty * transform.matrix.d;
                    size.0 = f32::max(size.0, tx + (glyph.advance as f32));
                    size.1 = f32::max(size.1, ty);
                },
            );
        }

        size
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.0.id
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        Some(*self)
    }

    fn post_instantiation(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Object<'gc>,
    ) {
        let mut text = self.0.write(gc_context);
        if text.object.is_none() {
            let object =
                StageObject::for_display_object(gc_context, display_object, Some(proto)).into();

            attach_virtual_properties(gc_context, object);

            text.object = Some(object);
        }
    }

    fn object(&self) -> Value<'gc> {
        self.0
            .read()
            .object
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&*self.transform());

        let text_transform = self.text_transform();

        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some(font) = context
            .library
            .get_font(font_id)
            .filter(|font| font.has_glyphs())
            .or_else(|| context.library.device_font())
        {
            let height = static_data
                .height
                .map(|v| v as f32)
                .unwrap_or_else(|| font.scale());

            font.evaluate(
                &edit_text.text,
                text_transform,
                height,
                edit_text.static_data.0.is_html,
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

        context.transform_stack.pop();
    }

    fn allow_as_mask(&self) -> bool {
        false
    }
}

unsafe impl<'gc> gc_arena::Collect for EditTextData<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
        self.object.trace(cc);
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EditTextStatic(swf::EditText);

unsafe impl<'gc> gc_arena::Collect for EditTextStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
