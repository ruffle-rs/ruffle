//! `flash.text.Font` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_1508;
use crate::avm2::object::{FontObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::string::AvmString;

pub use crate::avm2::object::font_allocator;
use crate::character::Character;
use crate::font::{Font, FontType};

use ruffle_macros::istr;

/// Implements `Font.fontName`
pub fn get_font_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(font) = this.as_font() {
        return Ok(AvmString::new_utf8(activation.gc(), font.descriptor().name()).into());
    }

    Ok(Value::Null)
}

/// Implements `Font.fontStyle`
pub fn get_font_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(font) = this.as_font() {
        let font_style = match (font.descriptor().bold(), font.descriptor().italic()) {
            (false, false) => istr!("regular"),
            (false, true) => istr!("italic"),
            (true, false) => istr!("bold"),
            (true, true) => istr!("boldItalic"),
        };

        return Ok(font_style.into());
    }

    Ok(Value::Null)
}

/// Implements `Font.fontType`
pub fn get_font_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(font) = this.as_font() {
        let font_type = match font.font_type() {
            FontType::Embedded => istr!("embedded"),
            FontType::EmbeddedCFF => istr!("embeddedCFF"),
            FontType::Device => istr!("device"),
        };

        return Ok(font_type.into());
    }

    Ok(Value::Null)
}

/// Implements `Font.hasGlyphs`
pub fn has_glyphs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(font) = this.as_font() {
        let my_str = args.get_string(activation, 0)?;
        return Ok(font.has_glyphs_for_str(&my_str).into());
    }

    Ok(Value::Bool(false))
}

/// `Font.enumerateFonts`
pub fn enumerate_fonts<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut fonts: Vec<Font<'gc>> = Vec::new();

    if args.get_bool(0) {
        // We could include the ones we know about, but what to do for the ones that weren't eagerly loaded?
        avm2_stub_method!(
            activation,
            "flash.text.Font",
            "enumerateFonts",
            "with device fonts"
        );
    }

    fonts.append(&mut activation.context.library.global_fonts());

    if let Some(library) = activation
        .context
        .library
        .library_for_movie(activation.caller_movie_or_root())
    {
        for font in library.embedded_fonts() {
            // TODO: EmbeddedCFF isn't supposed to show until it's been used (some kind of internal initialization method?)
            // Device is only supposed to show when arg0 is true - but that's supposed to be "all known" device fonts, not just loaded ones
            if font.has_layout() && font.font_type() == FontType::Embedded {
                fonts.push(font);
            }
        }
    }

    // The output from Flash is sorted by font name (case insensitive).
    // If two fonts have the same name (e.g. bold/italic variants),
    // the order is nondeterministic.
    fonts.sort_unstable_by(|a, b| {
        a.descriptor()
            .lowercase_name()
            .cmp(b.descriptor().lowercase_name())
    });

    let font_class = activation.avm2().classes().font;
    let mut storage = ArrayStorage::new(fonts.len());
    for font in fonts {
        storage.push(FontObject::for_font(activation.gc(), font_class, font).into());
    }
    Ok(ArrayObject::from_storage(activation, storage).into())
}

/// `Font.registerFont`
pub fn register_font<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let object = args.get_object(activation, 0, "font")?;

    if let Some(class) = object.as_class_object() {
        if let Some((movie, id)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class.inner_class_definition())
        {
            if let Some(lib) = activation.context.library.library_for_movie(movie) {
                if let Some(Character::Font(font)) = lib.character_by_id(id) {
                    activation.context.library.register_global_font(*font);
                    return Ok(Value::Undefined);
                }
            }
        }
    }

    Err(make_error_1508(activation, "font"))
}
