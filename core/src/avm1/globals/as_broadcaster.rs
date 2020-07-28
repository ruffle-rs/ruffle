//! ActionScript Broadcaster (AsBroadcaster)

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute::*;
use crate::avm1::{Object, ScriptObject, UpdateContext, Value};
use gc_arena::MutationContext;

pub fn add_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this.get("_listeners", activation, context)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length();
        let mut position = None;

        for i in 0..length {
            let other_listener = listeners.array_element(i);
            if new_listener == other_listener {
                position = Some(i);
                break;
            }
        }

        if position == None {
            listeners.set_length(context.gc_context, length + 1);
            listeners.set_array_element(length, new_listener, context.gc_context);
        }
    }

    Ok(true.into())
}

pub fn remove_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let old_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this.get("_listeners", activation, context)?;

    let mut removed = false;
    if let Value::Object(listeners) = listeners {
        let length = listeners.length();
        let mut position = None;

        for i in 0..length {
            let other_listener = listeners.array_element(i);
            if old_listener == other_listener {
                position = Some(i);
                break;
            }
        }

        if let Some(position) = position {
            if length > 0 {
                let new_length = length - 1;
                for i in position..new_length {
                    listeners.set_array_element(
                        i,
                        listeners.array_element(i + 1),
                        context.gc_context,
                    );
                }

                listeners.delete_array_element(new_length, context.gc_context);
                listeners.delete(activation, context.gc_context, &new_length.to_string());

                listeners.set_length(context.gc_context, new_length);

                removed = true;
            }
        }
    }

    Ok(removed.into())
}

pub fn broadcast_message<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let event_name_value = args.get(0).cloned().unwrap_or(Value::Undefined);
    let event_name = event_name_value.coerce_to_string(activation, context)?;
    let call_args = &args[1..];

    let listeners = this.get("_listeners", activation, context)?;

    if let Value::Object(listeners) = listeners {
        for i in 0..listeners.length() {
            let listener = listeners.array_element(i);

            if let Value::Object(listener) = listener {
                listener.call_method(&event_name, call_args, activation, context)?;
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn initialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        let broadcaster = val.coerce_to_object(activation, context);

        let listeners =
            ScriptObject::array(context.gc_context, Some(activation.avm.prototypes().array));

        broadcaster.define_value(
            context.gc_context,
            "_listeners",
            Value::Object(listeners.into()),
            DontEnum.into(),
        );

        let add_listener = this
            .get("addListener", activation, context)
            .unwrap_or(Value::Undefined);
        broadcaster.define_value(
            context.gc_context,
            "addListener",
            add_listener,
            DontEnum.into(),
        );

        let remove_listener = this
            .get("removeListener", activation, context)
            .unwrap_or(Value::Undefined);
        broadcaster.define_value(
            context.gc_context,
            "removeListener",
            remove_listener,
            DontEnum.into(),
        );

        let broadcast_message = this
            .get("broadcastMessage", activation, context)
            .unwrap_or(Value::Undefined);
        broadcaster.define_value(
            context.gc_context,
            "broadcastMessage",
            broadcast_message,
            DontEnum.into(),
        );
    }
    Ok(Value::Undefined)
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut as_broadcaster = ScriptObject::object(gc_context, proto);

    as_broadcaster.force_set_function(
        "initialize",
        initialize,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    as_broadcaster.force_set_function(
        "addListener",
        add_listener,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    as_broadcaster.force_set_function(
        "removeListener",
        remove_listener,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    as_broadcaster.force_set_function(
        "broadcastMessage",
        broadcast_message,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    as_broadcaster.into()
}
