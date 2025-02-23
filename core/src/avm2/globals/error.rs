use crate::avm2::activation::Activation;
pub use crate::avm2::object::error_allocator;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::TObject;

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation
        .avm2()
        .classes()
        .error
        .construct(activation, args)
}

pub fn get_stack_trace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(error) = this.as_error_object() {
        let call_stack = error.call_stack();
        if !call_stack.is_empty() {
            return Ok(AvmString::new(activation.gc(), error.display_full()).into());
        }
    }
    Ok(Value::Null)
}
