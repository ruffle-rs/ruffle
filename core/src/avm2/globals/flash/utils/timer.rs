//! `flash.utils.Timer` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_utils_timer as slots;
use crate::avm2::object::TObject;
pub use crate::avm2::object::timer_allocator;
use crate::avm2::value::Value;
use crate::timer::TimerCallback;

/// Implements `Timer.stop`
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let timer = this.as_timer().unwrap();

    if let Some(native_timer) = timer.timer() {
        activation.context.timers.remove_timer(native_timer);
        timer.set_timer(None, activation.gc());
    }

    Ok(Value::Undefined)
}

/// Implements `Timer.start`
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let timer = this.as_timer().unwrap();

    let delay = this.get_slot(slots::_DELAY).as_f64() as i32;

    if timer.timer().is_none() {
        let on_update = this
            .get_slot(slots::_ON_UPDATE_CLOSURE)
            .as_object()
            .and_then(|o| o.as_function_object())
            .expect("Internal function is function");

        // Note - we deliberately do *not* check if currentCount is less than repeatCount.
        // Calling 'start' on a timer that has currentCount >= repeatCount will tick exactly
        // once, and then stop immediately. This is handled by Timer.onUpdate
        let mc = activation.gc();
        let native_timer = activation.context.timers.add_timer_without_id(
            mc,
            TimerCallback::Avm2Callback {
                closure: Some(on_update),
                params: vec![],
            },
            delay,
            false,
        );
        timer.set_timer(Some(native_timer), mc);
    }
    Ok(Value::Undefined)
}

/// Implements `Timer.updateDelay`
pub fn update_delay<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let timer = this.as_timer().unwrap();

    let delay = this.get_slot(slots::_DELAY).as_f64() as i32;

    if let Some(native_timer) = timer.timer() {
        activation.context.timers.set_delay(native_timer, delay);
    }
    Ok(Value::Undefined)
}

/// Implements `Timer.running`'s getter
pub fn get_running<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let timer = this.as_object().unwrap().as_timer().unwrap();
    Ok(timer.timer().is_some().into())
}

/// Implements `Timer.clearTimer`
pub fn clear_timer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let timer = this.as_object().unwrap().as_timer().unwrap();
    timer.set_timer(None, activation.gc());
    Ok(Value::Undefined)
}
