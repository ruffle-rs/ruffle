use crate::avm1::object::xml_socket_object::XmlSocketObject;
use crate::avm1::object::TObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, ExecutionReason, Object, Value};
use crate::avm_warn;
use crate::context::UpdateContext;
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "connect" => method(connect);
    "close" => method(close);
    "send" => method(send);
    "onConnect" => method(on_connect; DONT_ENUM | DONT_DELETE);
    "onClose" => method(on_close; DONT_ENUM | DONT_DELETE);
    "onData" => method(on_data; DONT_ENUM | DONT_DELETE);
    "onXML" => method(on_xml; DONT_ENUM | DONT_DELETE);
};

/// XMLSocket constructor
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

fn connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let socket = match this.as_xml_socket() {
        Some(socket) => socket,
        None => return Ok(false.into()), // FIXME: should we throw when `this` isn't a XMSocket instance?
    };

    let host = args
        .get(1)
        .unwrap_or(&"localhost".into())
        .coerce_to_string(activation)?;

    let port = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_u16(activation)?;

    let UpdateContext {
        xml_sockets,
        navigator,
        gc_context,
        ..
    } = &mut activation.context;

    let registered =
        if let Some(handle) = xml_sockets.connect(*navigator, this, &host.to_utf8_lossy(), port) {
            if let Some(previous_handle) = socket.set_handle(*gc_context, handle) {
                xml_sockets.close(previous_handle); // avoid leaking sockets
            }
            true
        } else {
            false
        };

    Ok(registered.into())
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(handle) = this.as_xml_socket().and_then(|s| s.handle()) {
        activation.context.xml_sockets.close(handle);
    }

    Ok(Value::Undefined)
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(handle) = this.as_xml_socket().and_then(|s| s.handle()) {
        let data = args
            .get(0)
            .map(|v| v.coerce_to_string(activation))
            .unwrap_or_else(|| Ok(Default::default()))?
            .to_string()
            .into_bytes();

        activation.context.xml_sockets.send(handle, data);
    }

    Ok(Value::Undefined)
}

fn on_connect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn on_close<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op by default
    Ok(Value::Undefined)
}

fn on_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_constructor = activation.context.avm1.prototypes().xml_constructor;
    if let Ok(xml) = xml_constructor.construct(activation, args) {
        let _ = this.call_method(
            "onXML".into(),
            &[xml],
            activation,
            ExecutionReason::FunctionCall,
        )?;
    } else {
        avm_warn!(
            activation,
            "default XMLSocket.onData() received invalid XML; message ignored"
        );
    }
    Ok(Value::Undefined)
}

fn on_xml<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op by default
    Ok(Value::Undefined)
}

/// Construct the prototype for `XMLSocket`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_socket_proto = XmlSocketObject::empty(gc_context, Some(proto));
    let object = xml_socket_proto.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    xml_socket_proto
}
