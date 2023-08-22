use crate::avm2::activation::Activation;
pub use crate::avm2::object::error_allocator;
use crate::avm2::object::Object;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::TObject;

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation
        .avm2()
        .classes()
        .error
        .construct(activation, args)?
        .into())
}

pub fn get_stack_trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(error) = this.as_error_object() {
        let call_stack = error.call_stack();
        if !call_stack.is_empty() {
            return Ok(AvmString::new(activation.context.gc_context, error.display_full()?).into());
        }
    }
    Ok(Value::Null)
}
