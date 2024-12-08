//! `flash.utils.Timer` native methods

use crate::avm2::activation::Activation;
use crate::avm2::globals::slots::flash_utils_timer as slots;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::{Error, Object};
use crate::timer::TimerCallback;

/// Implements `Timer.stop`
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = this.get_slot(slots::_TIMER_ID).coerce_to_i32(activation)?;

    if id != -1 {
        activation.context.timers.remove(id);
        this.set_slot(slots::_TIMER_ID, (-1).into(), activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Timer.start`
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;

    let id = this.get_slot(slots::_TIMER_ID).coerce_to_i32(activation)?;

    let delay = this.get_slot(slots::_DELAY).coerce_to_number(activation)?;

    if id == -1 {
        let on_update = this
            .get_property(
                &Multiname::new(namespaces.flash_utils_internal, "onUpdate"),
                activation,
            )?
            .as_object()
            .expect("Internal function is object");

        // Note - we deliberately do *not* check if currentCount is less than repeatCount.
        // Calling 'start' on a timer that has currentCount >= repeatCount will tick exactly
        // once, and then stop immediately. This is handled by Timer.onUpdate
        let id = activation.context.timers.add_timer(
            TimerCallback::Avm2Callback {
                closure: on_update,
                params: vec![],
            },
            delay as _,
            false,
        );
        this.set_slot(slots::_TIMER_ID, id.into(), activation)?;
    }
    Ok(Value::Undefined)
}

/// Implements `Timer.updateDelay`
pub fn update_delay<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = this.get_slot(slots::_TIMER_ID).coerce_to_i32(activation)?;

    let delay = this.get_slot(slots::_DELAY).coerce_to_i32(activation)?;

    if id != -1 {
        activation.context.timers.set_delay(id, delay);
    }
    Ok(Value::Undefined)
}
