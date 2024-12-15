use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::display_object::TDisplayObject;

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
