//! ActionScript Broadcaster (AsBroadcaster)

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute::*;
use crate::avm1::{Object, ScriptObject, UpdateContext, Value};
use gc_arena::{Collect, MutationContext};

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct BroadcasterFunctions<'gc> {
    pub add_listener: Object<'gc>,
    pub remove_listener: Object<'gc>,
    pub broadcast_message: Object<'gc>,
}

impl<'gc> BroadcasterFunctions<'gc> {
    pub fn initialize(
        self,
        gc_context: MutationContext<'gc, '_>,
        broadcaster: Object<'gc>,
        array_proto: Object<'gc>,
    ) {
        initialize_internal(gc_context, broadcaster, self, array_proto);
    }
}

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

    broadcast_internal(activation, context, this, call_args, &event_name)
}

pub fn broadcast_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    call_args: &[Value<'gc>],
    method_name: &str,
) -> Result<Value<'gc>, Error<'gc>> {
    let listeners = this.get("_listeners", activation, context)?;

    if let Value::Object(listeners) = listeners {
        for i in 0..listeners.length() {
            let listener = listeners.array_element(i);

            if let Value::Object(listener) = listener {
                listener.call_method(method_name, call_args, activation, context)?;
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn initialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        let broadcaster = val.coerce_to_object(activation, context);
        initialize_internal(
            context.gc_context,
            broadcaster,
            activation.avm.broadcaster_functions,
            activation.avm.prototypes().array,
        );
    }
    Ok(Value::Undefined)
}

pub fn initialize_internal<'gc>(
    gc_context: MutationContext<'gc, '_>,
    broadcaster: Object<'gc>,
    functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) {
    let listeners = ScriptObject::array(gc_context, Some(array_proto));

    broadcaster.define_value(gc_context, "_listeners", listeners.into(), DontEnum.into());

    broadcaster.define_value(
        gc_context,
        "addListener",
        functions.add_listener.into(),
        DontEnum.into(),
    );

    broadcaster.define_value(
        gc_context,
        "removeListener",
        functions.remove_listener.into(),
        DontEnum.into(),
    );

    broadcaster.define_value(
        gc_context,
        "broadcastMessage",
        functions.broadcast_message.into(),
        DontEnum.into(),
    );
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> (BroadcasterFunctions<'gc>, Object<'gc>) {
    let mut as_broadcaster = ScriptObject::object(gc_context, proto);

    as_broadcaster.force_set_function(
        "initialize",
        initialize,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    let add_listener =
        FunctionObject::function(gc_context, Executable::Native(add_listener), fn_proto, None);
    as_broadcaster.define_value(
        gc_context,
        "addListener",
        add_listener.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    let remove_listener = FunctionObject::function(
        gc_context,
        Executable::Native(remove_listener),
        fn_proto,
        None,
    );
    as_broadcaster.define_value(
        gc_context,
        "removeListener",
        remove_listener.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    let broadcast_message = FunctionObject::function(
        gc_context,
        Executable::Native(broadcast_message),
        fn_proto,
        None,
    );
    as_broadcaster.define_value(
        gc_context,
        "broadcastMessage",
        broadcast_message.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    (
        BroadcasterFunctions {
            add_listener,
            remove_listener,
            broadcast_message,
        },
        as_broadcaster.into(),
    )
}
