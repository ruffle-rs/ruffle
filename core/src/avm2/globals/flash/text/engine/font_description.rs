use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2008;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::fte::{
    CffHintingValue, FontLookupValue, FontPostureValue, FontWeightValue, RenderingModeValue,
};

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
    Ok(this.font_weight().as_string(activation).into())
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
    this.set_font_weight(
        FontWeightValue::parse_string(&s)
            .ok_or_else(|| make_error_2008(activation, "fontWeight"))?,
    );
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
    Ok(this.font_posture().as_string(activation).into())
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
    this.set_font_posture(
        FontPostureValue::parse_string(&s)
            .ok_or_else(|| make_error_2008(activation, "fontPosture"))?,
    );
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
    Ok(this.font_lookup().as_string(activation).into())
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
    this.set_font_lookup(
        FontLookupValue::parse_string(&s)
            .ok_or_else(|| make_error_2008(activation, "fontLookup"))?,
    );
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
    Ok(this.rendering_mode().as_string(activation).into())
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
    this.set_rendering_mode(
        RenderingModeValue::parse_string(&s)
            .ok_or_else(|| make_error_2008(activation, "renderingMode"))?,
    );
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
    Ok(this.cff_hinting().as_string(activation).into())
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
    this.set_cff_hinting(
        CffHintingValue::parse_string(&s)
            .ok_or_else(|| make_error_2008(activation, "cffHinting"))?,
    );
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
