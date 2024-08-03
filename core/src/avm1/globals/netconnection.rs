use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::shared_object::{deserialize_value, serialize};
use crate::avm1::object::Object;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{
    Activation, ActivationIdentifier, Error, ExecutionReason, NativeObject, ScriptObject, TObject,
    Value,
};
use crate::avm1_stub;
use crate::context::{GcContext, UpdateContext};
use crate::net_connection::{NetConnectionHandle, NetConnections, ResponderCallback};
use crate::string::AvmString;
use flash_lso::packet::Header;
use flash_lso::types::Value as AMFValue;
use gc_arena::{Collect, Gc};
use ruffle_wstr::WStr;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct NetConnectionData {
    handle: Cell<Option<NetConnectionHandle>>,
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct NetConnection<'gc>(Gc<'gc, NetConnectionData>);

impl<'gc> NetConnection<'gc> {
    pub fn handle(&self) -> Option<NetConnectionHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        self.0.handle.replace(handle)
    }

    pub fn cast(value: Value<'gc>) -> Option<Self> {
        if let Value::Object(object) = value {
            if let NativeObject::NetConnection(net_connection) = object.native() {
                return Some(net_connection);
            }
        }
        None
    }

    pub fn on_status_event(
        context: &mut UpdateContext<'gc>,
        this: Object<'gc>,
        code: &'static str,
    ) -> Result<(), Error<'gc>> {
        let Some(root_clip) = context.stage.root_clip() else {
            tracing::warn!("Ignored NetConnection callback as there's no root movie");
            return Ok(());
        };
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[NetConnection connect]"),
            root_clip,
        );
        let constructor = activation.context.avm1.prototypes().object_constructor;
        let event = constructor
            .construct(&mut activation, &[])?
            .coerce_to_object(&mut activation);
        event.set("code", code.into(), &mut activation)?;
        event.set("level", "status".into(), &mut activation)?;
        this.call_method(
            "onStatus".into(),
            &[event.into()],
            &mut activation,
            ExecutionReason::Special,
        )?;
        Ok(())
    }

    // [NA] I have no idea why this is a thing. It's similar in AVM2 too.
    pub fn on_empty_status_event(
        context: &mut UpdateContext<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let Some(root_clip) = context.stage.root_clip() else {
            tracing::warn!("Ignored NetConnection callback as there's no root movie");
            return Ok(());
        };
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[NetConnection connect]"),
            root_clip,
        );
        this.call_method(
            "onStatus".into(),
            &[],
            &mut activation,
            ExecutionReason::Special,
        )?;
        Ok(())
    }

    pub fn send_callback(
        context: &mut UpdateContext<'gc>,
        responder: Object<'gc>,
        callback: ResponderCallback,
        message: &flash_lso::types::Value,
    ) -> Result<(), Error<'gc>> {
        let Some(root_clip) = context.stage.root_clip() else {
            tracing::warn!("Ignored NetConnection response as there's no root movie");
            return Ok(());
        };
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[NetConnection response]"),
            root_clip,
        );
        let method_name = match callback {
            ResponderCallback::Result => "onResult",
            ResponderCallback::Status => "onStatus",
        };
        let reader = flash_lso::read::Reader::default();
        let mut reference_cache = BTreeMap::default();
        let value = deserialize_value(
            &mut activation,
            message,
            &reader.amf0_decoder,
            &mut reference_cache,
        );
        responder.call_method(
            method_name.into(),
            &[value],
            &mut activation,
            ExecutionReason::Special,
        )?;
        Ok(())
    }
}

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let net_connection = NetConnection(Gc::new(
        activation.gc(),
        NetConnectionData {
            handle: Cell::new(None),
        },
    ));

    this.set_native(activation.gc(), NativeObject::NetConnection(net_connection));
    Ok(this.into())
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "isConnected" => property(is_connected);
    "protocol" => property(protocol);
    "uri" => property(uri);

    "addHeader" => method(add_header; DONT_ENUM | DONT_DELETE);
    "call" => method(call; DONT_ENUM | DONT_DELETE);
    "close" => method(close; DONT_ENUM | DONT_DELETE);
    "connect" => method(connect; DONT_ENUM | DONT_DELETE);
};

fn is_connected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(net_connection) = NetConnection::cast(this.into()) {
        return Ok(net_connection
            .handle()
            .map(|handle| activation.context.net_connections.is_connected(handle))
            .unwrap_or_default()
            .into());
    }
    Ok(Value::Undefined)
}

fn protocol<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(net_connection) = NetConnection::cast(this.into()) {
        return if let Some(protocol) = net_connection
            .handle()
            .and_then(|handle| activation.context.net_connections.get_protocol(handle))
        {
            Ok(protocol.into())
        } else {
            Ok(Value::Undefined)
        };
    }
    Ok(Value::Undefined)
}

fn uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(net_connection) = NetConnection::cast(this.into()) {
        return if let Some(uri) = net_connection
            .handle()
            .and_then(|handle| activation.context.net_connections.get_uri(handle))
        {
            Ok(Value::String(AvmString::new_utf8(activation.gc(), uri)))
        } else {
            Ok(Value::Undefined)
        };
    }
    Ok(Value::Undefined)
}

fn add_header<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(net_connection) = NetConnection::cast(this.into()) else {
        return Ok(Value::Undefined);
    };

    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let must_understand = args
        .get(1)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.swf_version());

    let value = serialize(activation, *args.get(2).unwrap_or(&Value::Null));

    if let Some(handle) = net_connection.handle() {
        activation.context.net_connections.set_header(
            handle,
            Header {
                name: name.to_string(),
                must_understand,
                value: Rc::new(value),
            },
        );
    }

    Ok(Value::Undefined)
}

fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(net_connection) = NetConnection::cast(this.into()) else {
        return Ok(Value::Undefined);
    };

    let command = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let mut arguments = Vec::new();

    for arg in &args[2..] {
        arguments.push(Rc::new(serialize(activation, *arg)));
    }

    if let Some(handle) = net_connection.handle() {
        if let Some(responder) = args.get(1) {
            let responder = responder.coerce_to_object(activation);
            NetConnections::send_avm1(
                activation.context,
                handle,
                command.to_string(),
                AMFValue::StrictArray(arguments),
                responder,
            );
        } else {
            NetConnections::send_without_response(
                activation.context,
                handle,
                command.to_string(),
                AMFValue::StrictArray(arguments),
            );
        }
    }

    Ok(Value::Undefined)
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(net_connection) = NetConnection::cast(this.into()) {
        if let Some(previous_handle) = net_connection.set_handle(None) {
            NetConnections::close(activation.context, previous_handle, true);
        }
    }
    Ok(Value::Undefined)
}

fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if matches!(
        args.get(0),
        None | Some(Value::Undefined) | Some(Value::Null)
    ) {
        NetConnections::connect_to_local(activation.context, this);
        return Ok(Value::Undefined);
    }

    let url = args[0].coerce_to_string(activation)?;
    if url.starts_with(WStr::from_units(b"http://"))
        || url.starts_with(WStr::from_units(b"https://"))
    {
        // HTTP(S) is for Flash Remoting, which is just POST requests to the URL.
        NetConnections::connect_to_flash_remoting(activation.context, this, url.to_string());
    } else {
        avm1_stub!(
            activation,
            "NetConnection",
            "connect",
            "with non-null, non-http command"
        );
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

pub fn create_class<'gc>(
    context: &mut GcContext<'_, 'gc>,
    netconnection_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        netconnection_proto,
    )
}
