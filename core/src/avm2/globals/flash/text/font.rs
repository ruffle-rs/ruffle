//! `flash.text.Font` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{FontObject, Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::string::AvmString;

pub use crate::avm2::object::font_allocator;
use crate::font::FontType;

/// Implements `Font.fontName`
pub fn get_font_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(font) = this.as_font() {
        return Ok(
            AvmString::new_utf8(activation.context.gc_context, font.descriptor().name()).into(),
        );
    }

    Ok(Value::Null)
}

/// Implements `Font.fontStyle`
pub fn get_font_style<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(font) = this.as_font() {
        return match (font.descriptor().bold(), font.descriptor().italic()) {
            (false, false) => Ok("regular".into()),
            (false, true) => Ok("italic".into()),
            (true, false) => Ok("bold".into()),
            (true, true) => Ok("boldItalic".into()),
        };
    }

    Ok(Value::Null)
}

/// Implements `Font.fontType`
pub fn get_font_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(font) = this.as_font() {
        return Ok(match font.font_type() {
            FontType::Embedded => "embedded",
            FontType::EmbeddedCFF => "embeddedCFF",
            FontType::Device => "device",
        }
        .into());
    }

    Ok(Value::Null)
}

/// Implements `Font.hasGlyphs`
pub fn has_glyphs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(font) = this.as_font() {
        let my_str = args.get_string(activation, 0)?;
        return Ok(font.has_glyphs_for_str(&my_str).into());
    }

    Ok(Value::Bool(false))
}

/// `Font.enumerateFonts`
pub fn enumerate_fonts<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut storage = ArrayStorage::new(0);
    let font_class = activation.avm2().classes().font;

    if args.get_bool(0) {
        // We could include the ones we know about, but what to do for the ones that weren't eagerly loaded?
        avm2_stub_method!(
            activation,
            "flash.text.Font",
            "enumerateFonts",
            "with device fonts"
        );
    }

    if let Some(library) = activation
        .context
        .library
        .library_for_movie(activation.context.swf.clone())
    {
        for font in library.embedded_fonts() {
            // TODO: EmbeddedCFF isn't supposed to show until it's been used (some kind of internal initialization method?)
            // Device is only supposed to show when arg0 is true - but that's supposed to be "all known" device fonts, not just loaded ones
            if font.font_type() == FontType::Embedded {
                storage.push(
                    FontObject::for_font(activation.context.gc_context, font_class, font).into(),
                );
            }
        }
    }

    Ok(ArrayObject::from_storage(activation, storage)?.into())
}

/// `Font.registerFont`
pub fn register_font<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.text.Font", "registerFont");
    Ok(Value::Undefined)
}
