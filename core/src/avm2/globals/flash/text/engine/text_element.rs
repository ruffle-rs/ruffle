use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::ElementData;
use crate::avm2::parameters::ParametersExt;
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
    *data = ElementData::Text { text: None };

    Ok(Value::Undefined)
}

pub fn set_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();
    let new_text = args.try_get_string(0);

    let mut data = this.element_data_mut(activation.gc());
    let ElementData::Text { text } = &mut *data else {
        unreachable!("Data can only have been set to Text");
    };
    *text = new_text;

    Ok(Value::Undefined)
}
