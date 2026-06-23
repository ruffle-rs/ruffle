use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;

pub use crate::avm2::object::content_element_allocator;

pub fn get_text<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(match this.text() {
        Some(s) => s.into(),
        None => Value::Null,
    })
}

pub fn get_element_format<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    Ok(match this.element_format() {
        Some(ef) => ef.into(),
        None => Value::Null,
    })
}

pub fn set_element_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    let ef = args
        .try_get_object(0)
        .and_then(|o| o.as_element_format_object());
    this.set_element_format(ef, activation.gc());
    Ok(Value::Undefined)
}
