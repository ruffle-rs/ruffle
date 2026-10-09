//! ActionScript Broadcaster (AsBroadcaster)
//!
//! Implemented in `AsBroadcaster.as`, except for `broadcastMessage`.

use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::{Activation, Object, Value};
use crate::string::AvmString;
use ruffle_macros::istr;

pub fn broadcast_message<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let event_name = args.try_get_string(activation, 0, UndefinedAs::Some)?;
    if let Some(event_name) = event_name {
        let call_args = &args[1..];
        let has_listeners = broadcast_internal(this, call_args, event_name, activation)?;
        if has_listeners {
            return Ok(true.into());
        }
    }

    Ok(Value::Undefined)
}

pub fn broadcast_internal<'gc>(
    this: Object<'gc>,
    call_args: &[Value<'gc>],
    method_name: AvmString<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<bool, Error<'gc>> {
    let listeners = this.get(istr!("_listeners"), activation)?;
    let Value::Object(listeners) = listeners else {
        return Ok(false);
    };

    // Collect the listeners up front, so that every listener present when the broadcast
    // starts gets called, even if `_listeners` is modified by a listener.
    let length = listeners.length(activation)?;
    let listeners: Vec<_> = (0..length)
        .map(|i| listeners.get_element(activation, i))
        .collect();

    let mut has_listeners = false;
    for listener in listeners {
        // Primitive listeners are boxed, so they get their prototype's methods.
        // `null` and `undefined` are skipped, and don't count as listeners.
        let Some(listener_object) = listener.coerce_to_object(activation)? else {
            continue;
        };

        has_listeners = true;
        if method_name.is_empty() {
            listener_object.call(method_name, activation, listener, call_args)?;
        } else {
            listener_object.call_method(
                method_name,
                call_args,
                activation,
                ExecutionReason::Special,
            )?;
        }
    }

    Ok(has_listeners)
}
