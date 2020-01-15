//! `MovieClipLoader` impl

use crate::avm1::object::TObject;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::script_object::ScriptObject;
use crate::avm1::{Avm1, Error, Object, UpdateContext, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let listeners = ScriptObject::array(context.gc_context, Some(avm.prototypes().array));
    this.define_value(
        context.gc_context,
        "_listeners",
        Value::Object(listeners.into()),
        Attribute::DontEnum.into(),
    );
    listeners.set("0", Value::Object(this), avm, context)?;

    Ok(Value::Undefined.into())
}

pub fn add_listener<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this
        .get("_listeners", avm, context)?
        .resolve(avm, context)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length();
        listeners.set_length(context.gc_context, length + 1);
        listeners.set_array_element(length, new_listener, context.gc_context);
    }

    Ok(true.into())
}

pub fn remove_listener<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this
        .get("_listeners", avm, context)?
        .resolve(avm, context)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length();
        let mut position = None;

        for i in 0..length {
            let other_listener = listeners
                .get(&format!("{}", i), avm, context)?
                .resolve(avm, context)?;
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
                listeners.delete(context.gc_context, &new_length.to_string());

                listeners.set_length(context.gc_context, new_length);
            }
        }
    }

    Ok(true.into())
}

pub fn broadcast_message<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let event_name = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(avm, context)?;
    let call_args = &args[0..];

    let listeners = this
        .get("_listeners", avm, context)?
        .resolve(avm, context)?;
    if let Value::Object(listeners) = listeners {
        for i in 0..listeners.length() {
            let listener = listeners
                .get(&format!("{}", i), avm, context)?
                .resolve(avm, context)?;

            if let Value::Object(listener) = listener {
                let handler = listener
                    .get(&event_name, avm, context)?
                    .resolve(avm, context)?;
                handler
                    .call(avm, context, listener, call_args)?
                    .resolve(avm, context)?;
            }
        }
    }

    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mcl_proto = ScriptObject::object(gc_context, Some(proto));

    mcl_proto.as_script_object().unwrap().force_set_function(
        "addListener",
        add_listener,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    mcl_proto.as_script_object().unwrap().force_set_function(
        "removeListener",
        remove_listener,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    mcl_proto.as_script_object().unwrap().force_set_function(
        "broadcastMessage",
        broadcast_message,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    mcl_proto.into()
}
