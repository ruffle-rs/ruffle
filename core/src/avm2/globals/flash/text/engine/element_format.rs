use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error2004Type, make_error_2004, make_error_2007, make_error_2008};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::fte::{
    BreakOpportunityValue, DigitCaseValue, DigitWidthValue, KerningValue, LigatureLevelValue,
    TextBaselineValue, TextRotationValue, TypographicCaseValue,
};

pub use crate::avm2::object::element_format_allocator;

pub fn get_alignment_baseline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.alignment_baseline().as_string(activation).into())
}

pub fn set_alignment_baseline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "alignmentBaseline")?;
    let Some(value) = TextBaselineValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "alignmentBaseline"));
    };
    this.set_alignment_baseline(value);
    Ok(Value::Undefined)
}

pub fn get_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.alpha().into())
}

pub fn set_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_alpha(args.get_f64(0));
    Ok(Value::Undefined)
}

pub fn get_baseline_shift<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.baseline_shift().into())
}

pub fn set_baseline_shift<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_baseline_shift(args.get_f64(0));
    Ok(Value::Undefined)
}

pub fn get_break_opportunity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.break_opportunity().as_string(activation).into())
}

pub fn set_break_opportunity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "breakOpportunity")?;
    let Some(value) = BreakOpportunityValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "breakOpportunity"));
    };
    this.set_break_opportunity(value);
    Ok(Value::Undefined)
}

pub fn get_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.color().to_rgba().into())
}

pub fn set_color<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_color(swf::Color::from_rgba(args.get_u32(0)));
    Ok(Value::Undefined)
}

pub fn get_digit_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.digit_case().as_string(activation).into())
}

pub fn set_digit_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "digitCase")?;
    let Some(value) = DigitCaseValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "digitCase"));
    };
    this.set_digit_case(value);
    Ok(Value::Undefined)
}

pub fn get_digit_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.digit_width().as_string(activation).into())
}

pub fn set_digit_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "digitWidth")?;
    let Some(value) = DigitWidthValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "digitWidth"));
    };
    this.set_digit_width(value);
    Ok(Value::Undefined)
}

pub fn get_dominant_baseline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.dominant_baseline().as_string(activation).into())
}

pub fn set_dominant_baseline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "dominantBaseline")?;
    let Some(value) = TextBaselineValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "dominantBaseline"));
    };
    if value == TextBaselineValue::UseDominantBaseline {
        return Err(make_error_2008(activation, "dominantBaseline"));
    }
    this.set_dominant_baseline(value);
    Ok(Value::Undefined)
}

pub fn get_font_description<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(match this.font_description() {
        Some(fd) => fd.into(),
        None => Value::Null,
    })
}

pub fn set_font_description<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let fd = args
        .get_value(0)
        .as_object()
        .and_then(|o| o.as_font_description_object());
    let Some(fd) = fd else {
        return Err(make_error_2007(activation, "fontDescription"));
    };
    this.set_font_description(fd, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_font_size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.font_size().into())
}

pub fn set_font_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let size = args.get_f64(0);
    if size < 0.0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }
    this.set_font_size(size);
    Ok(Value::Undefined)
}

pub fn get_kerning<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.kerning().as_string(activation).into())
}

pub fn set_kerning<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "kerning")?;
    let Some(value) = KerningValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "kerning"));
    };
    this.set_kerning(value);
    Ok(Value::Undefined)
}

pub fn get_ligature_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.ligature_level().as_string(activation).into())
}

pub fn set_ligature_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "ligatureLevel")?;
    let Some(value) = LigatureLevelValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "ligatureLevel"));
    };
    this.set_ligature_level(value);
    Ok(Value::Undefined)
}

pub fn get_locale<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.locale().into())
}

pub fn set_locale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let locale = args.get_string_non_null(activation, 0, "locale")?;
    this.set_locale(locale, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.text_rotation().as_string(activation).into())
}

pub fn set_text_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "textRotation")?;
    let Some(value) = TextRotationValue::parse_string(&s) else {
        return Err(make_error_2008(activation, "textRotation"));
    };
    this.set_text_rotation(value);
    Ok(Value::Undefined)
}

pub fn get_tracking_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.tracking_left().into())
}

pub fn set_tracking_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_tracking_left(args.get_f64(0));
    Ok(Value::Undefined)
}

pub fn get_tracking_right<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.tracking_right().into())
}

pub fn set_tracking_right<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_tracking_right(args.get_f64(0));
    Ok(Value::Undefined)
}

pub fn get_typographic_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.typographic_case().as_string(activation).into())
}

pub fn set_typographic_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "typographicCase")?;
    let Some(value) = TypographicCaseValue::parse_avm2_string(&s) else {
        return Err(make_error_2008(activation, "typographicCase"));
    };
    this.set_typographic_case(value);
    Ok(Value::Undefined)
}

pub fn get_locked<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    Ok(this.locked().into())
}

pub fn set_locked<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_element_format_object()
        .unwrap();
    this.set_locked(args.get_bool(0));
    Ok(Value::Undefined)
}
