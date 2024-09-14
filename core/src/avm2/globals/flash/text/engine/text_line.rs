use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::display_object::TDisplayObject;

pub fn super_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;
    Ok(Value::Undefined)
}

pub fn get_text_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.0.to_pixels().into())
}

pub fn get_text_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_object = this.as_display_object().unwrap();
    let edit_text = display_object.as_edit_text().unwrap();

    let measured_text = edit_text.measure_text(activation.context);
    Ok(measured_text.1.to_pixels().into())
}
