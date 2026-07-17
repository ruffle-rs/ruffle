//! `flash.system.Worker` native methods

use crate::avm2::Avm2;
use crate::avm2::Avm2StrRepresentable;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::{EventObject, MessageChannelObject, WorkerObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;

fn this_worker<'gc>(this: Value<'gc>) -> WorkerObject<'gc> {
    this.as_object()
        .and_then(|o| o.as_worker_object())
        .expect("Worker native called on non-Worker object")
}

/// Implements `Worker.state`
pub fn get_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let state = this_worker(this).state().as_avm2_str(activation);

    Ok(state.into())
}

/// Implements `Worker.isPrimordial`
pub fn get_is_primordial<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this_worker(this).is_primordial().into())
}

/// Implements `Worker.createMessageChannel`
pub fn create_message_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "createMessageChannel");

    let _receiver = args.get_object(activation, 0, "receiver")?;

    let message_channel = MessageChannelObject::new(activation);

    Ok(message_channel.into())
}

/// Implements `Worker.setSharedProperty`
pub fn set_shared_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "setSharedProperty");

    Ok(Value::Undefined)
}

/// Implements `Worker.getSharedProperty`
pub fn get_shared_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "getSharedProperty");

    Ok(Value::Undefined)
}

/// Implements `Worker.start`
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "start");

    let worker = this_worker(this);

    if worker.start() {
        let event = EventObject::bare_default_event(activation.context, "workerState");

        Avm2::dispatch_event(activation.context, event, worker.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Worker.terminate`
pub fn terminate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "terminate");

    let worker = this_worker(this);
    let changed = worker.terminate(activation)?;

    if changed {
        let event = EventObject::bare_default_event(activation.context, "workerState");

        Avm2::dispatch_event(activation.context, event, worker.into());
    }

    Ok(changed.into())
}

/// Implements `Worker.current`
pub fn get_current<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.avm2().current_worker().into())
}
