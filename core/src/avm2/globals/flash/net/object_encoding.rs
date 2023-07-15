use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{avm2_stub_getter, avm2_stub_setter};

pub fn get_dynamic_property_writer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.net.ObjectEncoding",
        "dynamicPropertyWriter"
    );
    Ok(Value::Undefined)
}

pub fn set_dynamic_property_writer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.net.ObjectEncoding",
        "dynamicPropertyWriter"
    );
    Ok(Value::Undefined)
}
