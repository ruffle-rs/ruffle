use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::TextLineLayout;
use std::cell::Ref;

fn text_line_layout<'gc>(this: Value<'gc>) -> Ref<'gc, TextLineLayout<'gc>> {
    this.as_object()
        .expect("TextLine native getter receiver must be an object")
        .as_display_object()
        .expect("TextLine native getter receiver must be a display object")
        .as_text_line()
        .expect("TextLine native getter receiver must be a TextLine")
        .line()
}

pub fn get_text_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.text_width() as f64).into())
}

pub fn get_text_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok(((line.ascent() + line.descent()) as f64).into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.ascent() as f64).into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let line = text_line_layout(this);
    Ok((line.descent() as f64).into())
}

pub fn get_baseline_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let baseline = args.get_value(0).coerce_to_string(activation)?;
    let (ascent, descent) = match fte_line(this) {
        Some(line) => (line.ascent(), line.descent()),
        None => (12.0, 3.0),
    };
    let position = match baseline.to_utf8_lossy().as_ref() {
        "roman" => 0.0,
        "ascent" => -ascent,
        "descent" => descent,
        "ideographicTop" => -ascent,
        "ideographicCenter" => (descent - ascent) / 2.0,
        "ideographicBottom" => descent,
        _ => 0.0,
    };
    Ok((position as f64).into())
}
