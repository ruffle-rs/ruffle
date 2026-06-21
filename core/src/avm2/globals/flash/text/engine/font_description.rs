use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::object::{
    CffHintingValue, FontLookupValue, FontPostureValue, FontWeightValue, RenderingModeValue,
};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use ruffle_macros::istr;

pub use crate::avm2::object::font_description_allocator;

pub fn get_font_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(this.font_name().into())
}

pub fn set_font_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let name = args.get_string_non_null(activation, 0, "fontName")?;
    this.set_font_name(name, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_font_weight<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(match this.font_weight() {
        FontWeightValue::Normal => istr!("normal").into(),
        FontWeightValue::Bold => istr!("bold").into(),
    })
}

pub fn set_font_weight<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "fontWeight")?;
    this.set_font_weight(if &s == b"normal" {
        FontWeightValue::Normal
    } else if &s == b"bold" {
        FontWeightValue::Bold
    } else {
        return Err(make_error_2008(activation, "fontWeight"));
    });
    Ok(Value::Undefined)
}

pub fn get_font_posture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(match this.font_posture() {
        FontPostureValue::Normal => istr!("normal").into(),
        FontPostureValue::Italic => istr!("italic").into(),
    })
}

pub fn set_font_posture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "fontPosture")?;
    this.set_font_posture(if &s == b"normal" {
        FontPostureValue::Normal
    } else if &s == b"italic" {
        FontPostureValue::Italic
    } else {
        return Err(make_error_2008(activation, "fontPosture"));
    });
    Ok(Value::Undefined)
}

pub fn get_font_lookup<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(match this.font_lookup() {
        FontLookupValue::Device => istr!("device").into(),
        FontLookupValue::EmbeddedCFF => istr!("embeddedCFF").into(),
    })
}

pub fn set_font_lookup<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "fontLookup")?;
    this.set_font_lookup(if &s == b"device" {
        FontLookupValue::Device
    } else if &s == b"embeddedCFF" {
        FontLookupValue::EmbeddedCFF
    } else {
        return Err(make_error_2008(activation, "fontLookup"));
    });
    Ok(Value::Undefined)
}

pub fn get_rendering_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(match this.rendering_mode() {
        RenderingModeValue::Normal => istr!("normal").into(),
        RenderingModeValue::Cff => istr!("cff").into(),
    })
}

pub fn set_rendering_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "renderingMode")?;
    this.set_rendering_mode(if &s == b"normal" {
        RenderingModeValue::Normal
    } else if &s == b"cff" {
        RenderingModeValue::Cff
    } else {
        return Err(make_error_2008(activation, "renderingMode"));
    });
    Ok(Value::Undefined)
}

pub fn get_cff_hinting<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    Ok(match this.cff_hinting() {
        CffHintingValue::None => istr!("none").into(),
        CffHintingValue::HorizontalStem => istr!("horizontalStem").into(),
    })
}

pub fn set_cff_hinting<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_font_description_object()
        .unwrap();
    let s = args.get_string_non_null(activation, 0, "cffHinting")?;
    this.set_cff_hinting(if &s == b"none" {
        CffHintingValue::None
    } else if &s == b"horizontalStem" {
        CffHintingValue::HorizontalStem
    } else {
        return Err(make_error_2008(activation, "cffHinting"));
    });
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
        .as_font_description_object()
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
        .as_font_description_object()
        .unwrap();
    this.set_locked(args.get_bool(0));
    Ok(Value::Undefined)
}
