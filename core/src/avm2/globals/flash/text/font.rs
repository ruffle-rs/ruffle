//! `flash.text.Font` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::character::Character;
use crate::string::AvmString;

/// Implements `Font.fontName`
pub fn get_font_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some((movie, character_id)) = this.instance_of().and_then(|this| {
        activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(this)
    }) {
        if let Some(Character::Font(font)) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_id(character_id)
        {
            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                font.descriptor().class(),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Font.fontStyle`
pub fn get_font_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some((movie, character_id)) = this.instance_of().and_then(|this| {
        activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(this)
    }) {
        if let Some(Character::Font(font)) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_id(character_id)
        {
            return match (font.descriptor().bold(), font.descriptor().italic()) {
                (false, false) => Ok("regular".into()),
                (false, true) => Ok("italic".into()),
                (true, false) => Ok("bold".into()),
                (true, true) => Ok("boldItalic".into()),
            };
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Font.fontType`
pub fn get_font_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some((movie, character_id)) = this.instance_of().and_then(|this| {
        activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(this)
    }) {
        if let Some(Character::Font(_)) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_id(character_id)
        {
            //TODO: How do we distinguish between CFF and non-CFF embedded fonts?
            return Ok("embedded".into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Font.hasGlyphs`
pub fn has_glyphs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some((movie, character_id)) = this.instance_of().and_then(|this| {
        activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(this)
    }) {
        let my_str = args.get_string(activation, 0)?;

        if let Some(Character::Font(font)) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_id(character_id)
        {
            return Ok(font.has_glyphs_for_str(&my_str).into());
        }
    }

    Ok(Value::Undefined)
}

/// `Font.enumerateFonts`
pub fn enumerate_fonts<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.text.Font", "enumerateFonts");
    Ok(ArrayObject::from_storage(activation, ArrayStorage::new(0))?.into())
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
