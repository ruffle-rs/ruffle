//! `flash.text.engine.FontDescription` native methods

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::font::FontType;

/// Implements `FontDescription.isFontCompatible`.
pub fn is_font_compatible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let font_name = args.get_string(activation, 0);
    let font_weight = args.get_string(activation, 1);
    let font_posture = args.get_string(activation, 2);

    let is_bold = font_weight.to_utf8_lossy() == "bold";
    let is_italic = font_posture.to_utf8_lossy() == "italic";
    let movie = activation.caller_movie_or_root();
    let font_name = font_name.to_utf8_lossy();
    let font_name = font_name.trim();

    if font_name.is_empty() {
        return Ok(false.into());
    }

    let font = activation.context.library.get_embedded_font_by_name(
        font_name,
        FontType::EmbeddedCFF,
        is_bold,
        is_italic,
        Some(movie),
    );
    let is_compatible = match font {
        Some(font) => font.has_layout(),
        None => false,
    };

    Ok(is_compatible.into())
}
