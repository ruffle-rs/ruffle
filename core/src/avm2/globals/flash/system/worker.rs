//! `flash.system.Worker` native methods

use crate::avm2::Avm2StrRepresentable;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::{MessageChannelObject, WorkerObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::context::UpdateContext;

fn this_worker<'gc>(this: Value<'gc>) -> WorkerObject<'gc> {
    this.as_object()
        .and_then(|o| o.as_worker_object())
        .expect("Worker native called on non-Worker object")
}

/// Implements `Worker.state`
pub fn get_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let state = this_worker(this).state().as_avm2_str(activation);

    Ok(state.into())
}

/// Implements `Worker.isPrimordial`
pub fn get_is_primordial<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this_worker(this).is_primordial().into())
}

/// Implements `Worker.createMessageChannel`
pub fn create_message_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
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
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "setSharedProperty");

    Ok(Value::Undefined)
}

/// Implements `Worker.getSharedProperty`
pub fn get_shared_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "getSharedProperty");

    Ok(Value::Undefined)
}

/// Implements `Worker.start`
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "start");

    let worker = this_worker(this);

    worker.start(activation)?;

    activation.avm2().pending_worker_updates.push(worker);

    Ok(Value::Undefined)
}

/// Implements `Worker.terminate`
pub fn terminate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Worker", "terminate");

    let worker = this_worker(this);
    let changed = worker.terminate(activation)?;

    if changed {
        activation.avm2().pending_worker_updates.push(worker);
    }

    Ok(changed.into())
}

/// Implements `Worker.current`
pub fn get_current<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.avm2().current_worker().into())
}

/// Applies the state changes of workers queued by `Worker.start` and
/// `Worker.terminate`, and dispatches their `workerState` events. Workers
/// queued by those event handlers are left for the next call.
pub fn update_pending_workers(context: &mut UpdateContext<'_>) {
    for worker in std::mem::take(&mut context.avm2.pending_worker_updates) {
        worker.update_state(context);
    }
}
