use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;

pub use crate::avm2::object::proxy_allocator;
use crate::avm2_stub_method;

pub fn is_attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // yes, this is supposed to be implemented
    avm2_stub_method!(activation, "flash.utils.Proxy", "isAttribute");
    Ok(Value::Undefined)
}
