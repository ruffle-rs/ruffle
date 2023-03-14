//! `flash.events.EventDispatcher` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::events::{dispatch_event as dispatch_event_internal, parent_of};
use crate::avm2::object::{DispatchObject, Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::{Avm2, Error};

/// Get an object's dispatch list, lazily initializing it if necessary.
fn dispatch_list<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mut this: Object<'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    match this.get_property(
        &Multiname::new(activation.avm2().flash_events_internal, "_dispatchList"),
        activation,
    )? {
        Value::Object(o) => Ok(o),
        _ => {
            let dispatch_list = DispatchObject::empty_list(activation.context.gc_context);
            this.init_property(
                &Multiname::new(activation.avm2().flash_events_internal, "_dispatchList"),
                dispatch_list.into(),
                activation,
            )?;

            Ok(dispatch_list)
        }
    }
}

/// Implements `EventDispatcher.addEventListener`.
pub fn add_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let dispatch_list = dispatch_list(activation, this)?;
        let event_type = args.get_string(activation, 0)?;
        let listener = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_callable(activation, None, None)?;
        let use_capture = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Bool(false))
            .coerce_to_boolean();
        let priority = args
            .get(3)
            .cloned()
            .unwrap_or(Value::Integer(0))
            .coerce_to_i32(activation)?;

        //TODO: If we ever get weak GC references, we should respect `useWeakReference`.
        dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .add_event_listener(event_type, priority, listener, use_capture);

        Avm2::register_broadcast_listener(&mut activation.context, this, event_type);
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.removeEventListener`.
pub fn remove_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let dispatch_list = dispatch_list(activation, this)?;
        let event_type = args.get_string(activation, 0)?;
        let listener = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_callable(activation, None, None)?;
        let use_capture = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Bool(false))
            .coerce_to_boolean();

        dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .remove_event_listener(event_type, listener, use_capture);
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.hasEventListener`.
pub fn has_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let dispatch_list = dispatch_list(activation, this)?;
        let event_type = args.get_string(activation, 0)?;

        return Ok(dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .has_event_listener(event_type)
            .into());
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.willTrigger`.
pub fn will_trigger<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let dispatch_list = dispatch_list(activation, this)?;
        let event_type = args.get_string(activation, 0)?;

        if dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .has_event_listener(event_type)
        {
            return Ok(true.into());
        }

        let target = this
            .get_property(
                &Multiname::new(activation.avm2().flash_events_internal, "_target"),
                activation,
            )?
            .as_object()
            .unwrap_or(this);

        if let Some(parent) = parent_of(target) {
            return will_trigger(activation, Some(parent), args);
        }
    }

    Ok(false.into())
}

/// Implements `EventDispatcher.dispatchEvent`.
pub fn dispatch_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let event = args.get(0).cloned().unwrap_or(Value::Undefined).as_object();

    if event.map(|o| o.as_event().is_none()).unwrap_or(true) {
        return Err("Dispatched Events must be subclasses of Event.".into());
    }

    if let Some(this) = this {
        Ok(dispatch_event_internal(activation, this, event.unwrap())?.into())
    } else {
        Ok(false.into())
    }
}

/// Implements `EventDispatcher.toString`.
///
/// This is an undocumented function, but MX will VerifyError if this isn't
/// present.
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let object_proto = activation.avm2().classes().object.prototype();
    let name = Multiname::new(activation.avm2().public_namespace, "toString");
    object_proto
        .get_property(&name, activation)?
        .as_callable(activation, Some(&name), Some(object_proto))?
        .call(this, args, activation)
}
