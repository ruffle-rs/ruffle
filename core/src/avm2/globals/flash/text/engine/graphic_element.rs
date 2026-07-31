use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::object::ElementData;
use crate::avm2::value::Value;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let mut data = this.element_data_mut(activation.gc());
    *data = ElementData::Graphic;

    Ok(Value::Undefined)
}
