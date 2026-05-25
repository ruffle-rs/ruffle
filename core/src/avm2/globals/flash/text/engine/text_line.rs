use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let ascent = edit_text
        .line_metrics(0)
        .map(|m| m.ascent.to_pixels())
        .unwrap_or(0.0);
    Ok(ascent.into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let descent = edit_text
        .line_metrics(0)
        .map(|m| m.descent.to_pixels())
        .unwrap_or(0.0);
    Ok(descent.into())
}

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.0.to_pixels().into())
}

pub fn get_text_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.1.to_pixels().into())
}
