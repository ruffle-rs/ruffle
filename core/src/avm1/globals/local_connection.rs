//! LocalConnection class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::shared_object::{deserialize_value, serialize};
use crate::avm1::object::TObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{
    ActivationIdentifier, ExecutionReason, NativeObject, Object, ScriptObject, Value,
};
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;
use crate::local_connection::{LocalConnectionHandle, LocalConnections};
use crate::string::{AvmString, StringContext};
use flash_lso::types::Value as AmfValue;
use gc_arena::{Collect, Gc};
use std::cell::RefCell;

#[derive(Debug, Collect)]
#[collect(require_static)]
struct LocalConnectionData {
    handle: RefCell<Option<LocalConnectionHandle>>,
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LocalConnection<'gc>(Gc<'gc, LocalConnectionData>);

impl<'gc> LocalConnection<'gc> {
    pub fn cast(value: Value<'gc>) -> Option<Self> {
        if let Value::Object(object) = value {
            if let NativeObject::LocalConnection(local_connection) = object.native() {
                return Some(local_connection);
            }
        }
        None
    }

    pub fn is_connected(&self) -> bool {
        self.0.handle.borrow().is_some()
    }

    pub fn connect(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        this: Object<'gc>,
    ) -> bool {
        if self.is_connected() {
            return false;
        }

        let connection_handle = activation.context.local_connections.connect(
            &LocalConnections::get_domain(activation.context.swf.url()),
            this,
            &name,
        );
        let result = connection_handle.is_some();
        *self.0.handle.borrow_mut() = connection_handle;
        result
    }

    pub fn disconnect(&self, activation: &mut Activation<'_, 'gc>) {
        if let Some(conn_handle) = self.0.handle.take() {
            activation.context.local_connections.close(conn_handle);
        }
    }

    pub fn send_status(
        context: &mut UpdateContext<'gc>,
        this: Object<'gc>,
        status: &'static str,
    ) -> Result<(), Error<'gc>> {
        let Some(root_clip) = context.stage.root_clip() else {
            tracing::warn!("Ignored LocalConnection callback as there's no root movie");
            return Ok(());
        };
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[LocalConnection onStatus]"),
            root_clip,
        );
        let constructor = activation.context.avm1.prototypes().object_constructor;
        let event = constructor
            .construct(&mut activation, &[])?
            .coerce_to_object(&mut activation);
        event.set("level", status.into(), &mut activation)?;
        this.call_method(
            "onStatus".into(),
            &[event.into()],
            &mut activation,
            ExecutionReason::Special,
        )?;
        Ok(())
    }

    pub fn run_method(
        context: &mut UpdateContext<'gc>,
        this: Object<'gc>,
        method_name: AvmString<'gc>,
        amf_arguments: Vec<AmfValue>,
    ) -> Result<(), Error<'gc>> {
        let Some(root_clip) = context.stage.root_clip() else {
            tracing::warn!("Ignored LocalConnection callback as there's no root movie");
            return Ok(());
        };
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[LocalConnection call]"),
            root_clip,
        );
        let mut args = Vec::with_capacity(amf_arguments.len());
        for arg in amf_arguments {
            let reader = flash_lso::read::Reader::default();
            let value = deserialize_value(
                &mut activation,
                &arg,
                &reader.amf0_decoder,
                &mut Default::default(),
            );
            args.push(value);
        }
        this.call_method(
            method_name,
            &args,
            &mut activation,
            ExecutionReason::Special,
        )?;
        Ok(())
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "domain" => method(domain; DONT_DELETE | DONT_ENUM);
    "connect" => method(connect; DONT_DELETE | DONT_ENUM);
    "close" => method(close; DONT_DELETE | DONT_ENUM);
    "send" => method(send; DONT_DELETE | DONT_ENUM);
};

pub fn domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.base_clip().movie();
    let domain = LocalConnections::get_domain(movie.url());

    Ok(Value::String(AvmString::new_utf8(
        activation.context.gc_context,
        domain,
    )))
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(Value::String(connection_name)) = args.get(0) else {
        // This is deliberately not a coercion, Flash tests the type
        return Ok(false.into());
    };
    if connection_name.is_empty() || connection_name.contains(b':') {
        return Ok(false.into());
    }

    if let Some(local_connection) = LocalConnection::cast(this.into()) {
        return Ok(local_connection
            .connect(activation, *connection_name, this)
            .into());
    }

    Ok(Value::Undefined)
}

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(Value::String(connection_name)) = args.get(0) else {
        // This is deliberately not a coercion, Flash tests the type
        return Ok(false.into());
    };
    let Some(Value::String(method_name)) = args.get(1) else {
        // This is deliberately not a coercion, Flash tests the type
        return Ok(false.into());
    };

    if connection_name.is_empty() || method_name.is_empty() {
        return Ok(false.into());
    }

    if method_name == b"send"
        || method_name == b"connect"
        || method_name == b"close"
        || method_name == b"allowDomain"
        || method_name == b"allowInsecureDomain"
        || method_name == b"domain"
    {
        return Ok(false.into());
    }

    let mut amf_arguments = Vec::with_capacity(args.len() - 2);
    for arg in &args[2..] {
        amf_arguments.push(serialize(activation, *arg));
    }

    activation.context.local_connections.send(
        &LocalConnections::get_domain(activation.context.swf.url()),
        this,
        *connection_name,
        *method_name,
        amf_arguments,
    );
    Ok(true.into())
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(local_connection) = LocalConnection::cast(this.into()) {
        local_connection.disconnect(activation);
    }
    Ok(Value::Undefined)
}

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_native(
        activation.gc(),
        NativeObject::LocalConnection(LocalConnection(Gc::new(
            activation.gc(),
            LocalConnectionData {
                handle: RefCell::new(None),
            },
        ))),
    );
    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
