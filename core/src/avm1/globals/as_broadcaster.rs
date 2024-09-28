//! ActionScript Broadcaster (AsBroadcaster)

use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::TObject;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::Declaration;
use crate::avm1::{Activation, ArrayObject, Object, ScriptObject, Value};
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, Mutation};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "initialize" => method(initialize; DONT_ENUM | DONT_DELETE);
    "addListener" => function(add_listener; DONT_ENUM | DONT_DELETE);
    "removeListener" => function(remove_listener; DONT_ENUM | DONT_DELETE);
    "broadcastMessage" => function(broadcast_message; DONT_ENUM | DONT_DELETE);
};

pub fn create<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> (BroadcasterFunctions<'gc>, Object<'gc>) {
    let gc_context = context.gc_context;
    let as_broadcaster_proto = ScriptObject::new(gc_context, Some(proto));
    let as_broadcaster = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        as_broadcaster_proto.into(),
    );
    let object = as_broadcaster.raw_script_object();

    let mut define_as_object = |index: usize| -> Object<'gc> {
        match OBJECT_DECLS[index].define_on(context, object, fn_proto) {
            Value::Object(o) => o,
            _ => panic!("expected object for broadcaster function"),
        }
    };

    define_as_object(0);
    (
        BroadcasterFunctions {
            add_listener: define_as_object(1),
            remove_listener: define_as_object(2),
            broadcast_message: define_as_object(3),
        },
        as_broadcaster,
    )
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct BroadcasterFunctions<'gc> {
    pub add_listener: Object<'gc>,
    pub remove_listener: Object<'gc>,
    pub broadcast_message: Object<'gc>,
}

impl<'gc> BroadcasterFunctions<'gc> {
    pub fn initialize(
        self,
        gc_context: &Mutation<'gc>,
        broadcaster: Object<'gc>,
        array_proto: Object<'gc>,
    ) {
        initialize_internal(gc_context, broadcaster, self, array_proto);
    }
}

fn add_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this.get("_listeners", activation)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        let exists = (0..length).any(|i| listeners.get_element(activation, i) == new_listener);
        if !exists {
            listeners.call_method(
                "push".into(),
                &[new_listener],
                activation,
                ExecutionReason::FunctionCall,
            )?;
        }
    }

    Ok(true.into())
}

fn remove_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let old_listener = args.get(0).cloned().unwrap_or(Value::Undefined);
    let listeners = this.get("_listeners", activation)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        if let Some(index) =
            (0..length).find(|&i| listeners.get_element(activation, i) == old_listener)
        {
            listeners.call_method(
                "splice".into(),
                &[index.into(), 1.into()],
                activation,
                ExecutionReason::FunctionCall,
            )?;
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

fn broadcast_message<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(event_name_value) = args.get(0) {
        let event_name = event_name_value.coerce_to_string(activation)?;
        let call_args = &args[1..];

        broadcast_internal(activation, this, call_args, event_name)?;
    }

    Ok(Value::Undefined)
}

pub fn broadcast_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    call_args: &[Value<'gc>],
    method_name: AvmString<'gc>,
) -> Result<bool, Error<'gc>> {
    let listeners = this.get("_listeners", activation)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        for i in 0..length {
            let listener = listeners.get_element(activation, i);

            if let Value::Object(listener) = listener {
                listener.call_method(
                    method_name,
                    call_args,
                    activation,
                    ExecutionReason::Special,
                )?;
            } else if let Value::MovieClip(_) = listener {
                let object = listener.coerce_to_object(activation);
                object.call_method(method_name, call_args, activation, ExecutionReason::Special)?;
            }
        }

        Ok(length > 0)
    } else {
        Ok(false)
    }
}

/// Implements `AsBroadcaster` constructor and function.
// Despite the documentation says that there is no constructor function for the `AsBroadcaster`
// class, Flash accepts expressions like `new AsBroadcaster()`, and a newly-created object is
// returned in such cases.
fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

fn initialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        let broadcaster = val.coerce_to_object(activation);
        initialize_internal(
            activation.context.gc_context,
            broadcaster,
            activation.context.avm1.broadcaster_functions(),
            activation.context.avm1.prototypes().array,
        );
    }
    Ok(Value::Undefined)
}

fn initialize_internal<'gc>(
    gc_context: &Mutation<'gc>,
    broadcaster: Object<'gc>,
    functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) {
    broadcaster.define_value(
        gc_context,
        "_listeners",
        ArrayObject::empty_with_proto(gc_context, array_proto).into(),
        Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        gc_context,
        "addListener",
        functions.add_listener.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        gc_context,
        "removeListener",
        functions.remove_listener.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        gc_context,
        "broadcastMessage",
        functions.broadcast_message.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
}
