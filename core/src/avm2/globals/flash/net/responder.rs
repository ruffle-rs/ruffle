pub use crate::avm2::object::responder_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let responder = this.as_responder().expect("Must be Responder object");

    let result = args.try_get_object(0);
    let status = args.try_get_object(1);

    responder.set_callbacks(
        activation.gc(),
        result.and_then(|o| o.as_function_object()),
        status.and_then(|o| o.as_function_object()),
    );

    Ok(Value::Undefined)
}
