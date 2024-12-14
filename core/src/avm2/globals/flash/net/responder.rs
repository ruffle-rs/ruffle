pub use crate::avm2::object::responder_allocator;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, Value};

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let responder = this.as_responder().expect("Must be Responder object");

    let result = args.get_object(activation, 0, "result")?;
    let status = args.try_get_object(activation, 1);

    responder.set_callbacks(
        activation.context.gc_context,
        result.as_function_object(),
        status.and_then(|o| o.as_function_object()),
    );

    Ok(Value::Undefined)
}
