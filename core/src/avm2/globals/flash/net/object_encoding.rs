use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn get_dynamic_property_writer<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("ObjectEncoding.dynamicPropertyWriter: Not yet implemented");
    Ok(Value::Undefined)
}

pub fn set_dynamic_property_writer<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("ObjectEncoding.dynamicPropertyWriter: Not yet implemented");
    Ok(Value::Undefined)
}
