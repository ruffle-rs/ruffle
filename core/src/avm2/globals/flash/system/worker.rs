//! `flash.system.Worker` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::{MessageChannelObject, WorkerObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;

/// Implements `Worker.createMessageChannel`
pub fn create_message_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let _receiver = args.get_object(activation, 0, "receiver")?;

    let message_channel = MessageChannelObject::new(activation);

    Ok(message_channel.into())
}

pub fn instantiate_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let worker = WorkerObject::new(activation);

    Ok(worker.into())
}
