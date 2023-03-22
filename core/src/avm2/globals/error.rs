use crate::avm2::activation::Activation;
pub use crate::avm2::object::error_allocator;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::TObject;

pub fn get_stack_trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(error) = this.and_then(|this| this.as_error_object()) {
        return Ok(error.display_full(activation)?.into());
    }
    Ok(Value::Undefined)
}
