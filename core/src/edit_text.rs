//! `EditText` display object and support code.
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::transform::Transform;

/// A dynamic text field.
/// The text in this text field can be changed dynamically.
/// It may be selectable or editable by the user, depending on the text field properties.
///
/// In the Flash IDE, this is created by changing the text field type to "Dynamic".
/// In AS2, this is created using `MovieClip.createTextField`.
/// In AS3, this is created with the `TextField` class. (https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html)
///
/// (SWF19 DefineEditText pp. 171-174)
#[derive(Clone, Debug)]
pub struct EditText<'gc> {
    /// DisplayObject common properties.
    base: DisplayObjectBase<'gc>,

    /// Static data shared among all instances of this `EditText`.
    static_data: gc_arena::Gc<'gc, EditTextStatic>,

    /// The current text displayed by this text field.
    text: String,
}

impl<'gc> EditText<'gc> {
    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(context: &mut UpdateContext<'_, 'gc, '_>, swf_tag: swf::EditText) -> Self {
        Self {
            base: DisplayObjectBase::new(context.swf_version),
            text: swf_tag.initial_text.clone().unwrap_or_default(),
            static_data: gc_arena::Gc::allocate(context.gc_context, EditTextStatic(swf_tag)),
        }
    }
}

impl<'gc> DisplayObject<'gc> for EditText<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.static_data.0.id
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        // TODO: This is a stub implementation to just get some dynamic text rendering.
        context.transform_stack.push(self.transform());
        let static_data = &self.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);
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
        let device_font = context.library.device_font();
        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        let font = context
            .library
            .get_font(font_id)
            .filter(|font| font.has_glyphs())
            .unwrap_or(device_font);
        let scale = if let Some(height) = static_data.height {
            transform.matrix.ty += f32::from(height);
            f32::from(height) / font.scale()
        } else {
            1.0
        };
        if let Some(layout) = &static_data.layout {
            transform.matrix.ty -= layout.leading.get() as f32;
        }
        transform.matrix.a = scale;
        transform.matrix.d = scale;
        let mut chars = self.text.chars().peekable();
        let has_kerning_info = font.has_kerning_info();
        while let Some(c) = chars.next() {
            // TODO: SWF text fields can contain a limited subset of HTML (and often do in SWF versions >6).
            // This is a quicky-and-dirty way to skip the HTML tags. This is obviously not correct
            // and we will need to properly parse and handle the HTML at some point.
            // See SWF19 pp. 173-174 for supported HTML tags.
            if self.static_data.0.is_html && c == '<' {
                // Skip characters until we see a close bracket.
                chars.by_ref().skip_while(|&x| x != '>').next();
            } else if let Some(glyph) = font.get_glyph_for_char(c) {
                // Render glyph.
                context.transform_stack.push(&transform);
                context
                    .renderer
                    .render_shape(glyph.shape, context.transform_stack.transform());
                context.transform_stack.pop();
                // Step horizontally.
                let mut advance = f32::from(glyph.advance);
                if has_kerning_info {
                    advance += font
                        .get_kerning_offset(c, chars.peek().cloned().unwrap_or('\0'))
                        .get() as f32;
                }
                transform.matrix.tx += advance * scale;
            }
        }
        context.transform_stack.pop();
    }
}

unsafe impl<'gc> gc_arena::Collect for EditText<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
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
