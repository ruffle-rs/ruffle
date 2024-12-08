//! `flash.events.EventDispatcher` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::events::{dispatch_event as dispatch_event_internal, parent_of};
use crate::avm2::globals::slots::flash_events_event_dispatcher as slots;
use crate::avm2::object::{DispatchObject, Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};

/// Get an object's dispatch list, lazily initializing it if necessary.
fn dispatch_list<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    match this.get_slot(slots::DISPATCH_LIST) {
        Value::Object(o) => Ok(o),
        _ => {
            let dispatch_list = DispatchObject::empty_list(activation);
            this.set_slot(slots::DISPATCH_LIST, dispatch_list.into(), activation)?;

            Ok(dispatch_list)
        }
    }
}

/// Implements `EventDispatcher.addEventListener`.
pub fn add_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dispatch_list = dispatch_list(activation, this)?;
    let event_type = args.get_string(activation, 0)?;
    let listener = args.get_object(activation, 1, "listener")?;
    let use_capture = args.get_bool(2);
    let priority = args.get_i32(activation, 3)?;

    //TODO: If we ever get weak GC references, we should respect `useWeakReference`.
    dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .expect("Internal properties should have what I put in them")
        .add_event_listener(event_type, priority, listener, use_capture);

    Avm2::register_broadcast_listener(activation.context, this, event_type);

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.removeEventListener`.
pub fn remove_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dispatch_list = dispatch_list(activation, this)?;
    let event_type = args.get_string(activation, 0)?;
    let listener = args.get_object(activation, 1, "listener")?;
    let use_capture = args.get_bool(2);

    dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .expect("Internal properties should have what I put in them")
        .remove_event_listener(event_type, listener, use_capture);

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.hasEventListener`.
pub fn has_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dispatch_list = dispatch_list(activation, this)?;
    let event_type = args.get_string(activation, 0)?;

    let does_have = dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .expect("Internal properties should have what I put in them")
        .has_event_listener(event_type)
        .into();

    Ok(does_have)
}

/// Implements `EventDispatcher.willTrigger`.
pub fn will_trigger<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dispatch_list = dispatch_list(activation, this)?;
    let event_type = args.get_string(activation, 0)?;

    if dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .expect("Internal properties should have what I put in them")
        .has_event_listener(event_type)
    {
        return Ok(true.into());
    }

    let target = this.get_slot(slots::TARGET).as_object().unwrap_or(this);

    if let Some(parent) = parent_of(target) {
        return will_trigger(activation, parent, args);
    }

    Ok(false.into())
}

/// Implements `EventDispatcher.dispatchEvent`.
pub fn dispatch_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let event = args.get_object(activation, 0, "event")?;

    if event.as_event().is_none() {
        return Err("Dispatched Events must be subclasses of Event.".into());
    }

    dispatch_event_internal(activation, this, event, false)?;
    let not_canceled = !event.as_event().unwrap().is_cancelled();
    Ok(not_canceled.into())
}
