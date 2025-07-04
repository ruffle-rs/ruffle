//! `flash.system.Worker` native methods

use crate::avm2::activation::Activation;
use crate::avm2::object::{MessageChannelObject, WorkerObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2_stub_method;

/// Implements `Worker.createMessageChannel`
pub fn create_message_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "createMessageChannel");

    let _received = args.get_object(activation, 0, "received")?;

    let message_channel = MessageChannelObject::new(activation);

    return Ok(Value::Object(message_channel.into()));
}

pub fn instantiate_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let worker = WorkerObject::new(activation);

    Ok(Value::Object(worker.into()))
}
