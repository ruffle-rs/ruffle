use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = this.as_display_object().unwrap();
    if let Some(fte_text_line) = display_object.as_fte_text_line() {
        if let Some(measured_text) = fte_text_line.measure_text(activation.context) {
            return Ok(measured_text.0.to_pixels().into());
        }
        return Ok(0.0.into());
    }
    let Some(edit_text) = display_object.as_edit_text() else {
        return Ok(0.0.into());
    };

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
    if let Some(fte_text_line) = display_object.as_fte_text_line() {
        if let Some(measured_text) = fte_text_line.measure_text(activation.context) {
            return Ok(measured_text.1.to_pixels().into());
        }
        return Ok(0.0.into());
    }
    let Some(edit_text) = display_object.as_edit_text() else {
        return Ok(0.0.into());
    };

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.1.to_pixels().into())
}
