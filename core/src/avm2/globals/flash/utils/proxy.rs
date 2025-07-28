use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub use crate::avm2::object::proxy_allocator;

pub fn is_attribute<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(Object::QNameObject(qname_object)) = args.get_value(0) {
        return Ok(qname_object.name().is_attribute().into());
    }
    Ok(false.into())
}
