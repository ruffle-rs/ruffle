//! ActionScript Broadcaster (AsBroadcaster)

use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, ArrayBuilder, Object, Value};
use crate::string::{AvmString, StringContext};
use gc_arena::Collect;
use ruffle_macros::istr;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "initialize" => method(initialize; DONT_ENUM | DONT_DELETE);
    "addListener" => function(add_listener; DONT_ENUM | DONT_DELETE);
    "removeListener" => function(remove_listener; DONT_ENUM | DONT_DELETE);
    "broadcastMessage" => function(broadcast_message; DONT_ENUM | DONT_DELETE);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> (BroadcasterFunctions<'gc>, SystemClass<'gc>) {
    // Despite the documentation says that there is no constructor function for the `AsBroadcaster`
    // class, Flash accepts expressions like `new AsBroadcaster()`, and a newly-created object is
    // returned in such cases.
    let class = context.empty_class(super_proto);

    let decls = OBJECT_DECLS(context);
    let mut define_as_object = |index: usize| -> Object<'gc> {
        match decls[index].define_on(context.strings, class.constr, context.fn_proto) {
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
        class,
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
        context: &StringContext<'gc>,
        broadcaster: Object<'gc>,
        array_proto: Object<'gc>,
    ) {
        initialize_internal(context, broadcaster, self, array_proto);
    }
}

fn add_listener<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_listener = args.get_value(0);
    let listeners = this.get(istr!("_listeners"), activation)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        let mut exists = false;
        for i in 0..length {
            if listeners
                .get_element(activation, i)
                .abstract_eq(new_listener, activation)?
            {
                listeners.set_element(activation, i, new_listener)?;
                exists = true;
                break;
            }
        }

        if !exists {
            listeners.call_method(
                istr!("push"),
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
    let old_listener = args.get_value(0);
    let listeners = this.get(istr!("_listeners"), activation)?;

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        for i in 0..length {
            if listeners
                .get_element(activation, i)
                .abstract_eq(old_listener, activation)?
            {
                listeners.call_method(
                    istr!("splice"),
                    &[i.into(), 1.into()],
                    activation,
                    ExecutionReason::FunctionCall,
                )?;
                return Ok(true.into());
            }
        }
    }

    Ok(false.into())
}

fn broadcast_message<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let event_name = args.try_get_string(activation, 0, UndefinedAs::Some)?;
    if let Some(event_name) = event_name {
        let call_args = &args[1..];
        broadcast_internal(this, call_args, event_name, activation)?;

        return Ok(true.into());
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

    if let Value::Object(listeners) = listeners {
        let length = listeners.length(activation)?;
        for i in 0..length {
            let listener = listeners.get_element(activation, i);
            if let Some(obj) = listener.as_object(activation) {
                if method_name.is_empty() {
                    obj.call(method_name, activation, listener, call_args)?;
                } else {
                    obj.call_method(method_name, call_args, activation, ExecutionReason::Special)?;
                }
            }
        }

        Ok(length > 0)
    } else {
        Ok(false)
    }
}

fn initialize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let broadcaster = args.get_object(activation, 0)?;
    initialize_internal(
        &activation.context.strings,
        broadcaster,
        activation
            .context
            .avm1
            .broadcaster_functions(activation.swf_version()),
        activation.prototypes().array,
    );
    Ok(Value::Undefined)
}

fn initialize_internal<'gc>(
    context: &StringContext<'gc>,
    broadcaster: Object<'gc>,
    functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) {
    broadcaster.define_value(
        context.gc(),
        istr!(context, "_listeners"),
        ArrayBuilder::new_with_proto(context, array_proto)
            .with([])
            .into(),
        Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        context.gc(),
        istr!(context, "addListener"),
        functions.add_listener.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        context.gc(),
        istr!(context, "removeListener"),
        functions.remove_listener.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
    broadcaster.define_value(
        context.gc(),
        istr!(context, "broadcastMessage"),
        functions.broadcast_message.into(),
        Attribute::DONT_DELETE | Attribute::DONT_ENUM,
    );
}
