use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::xml_socket::XmlSocket;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::avm_warn;
use crate::backend::navigator::ConnectOptions;
use gc_arena::Collect;
use gc_arena::MutationContext;
use std::collections::HashMap;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlSocketProperties<'gc> {
    pub sockets: HashMap<u64, Object<'gc>>,
    pub current_socket_id: u64,
}

impl<'gc> Default for XmlSocketProperties<'gc> {
    fn default() -> Self {
        XmlSocketProperties {
            sockets: HashMap::new(),
            current_socket_id: 0u64,
        }
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "connect" => method(connect);
    "close" => method(close);
    "send" => method(send);
    "onConnect" => property(get_on_connect, set_on_connect);
    "onClose" => property(get_on_close, set_on_close);
    "onData" => property(get_on_data, set_on_data);
    "onXML" => property(get_on_xml, set_on_xml);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {};

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.send");
    let s = if let Some(val) = args.get(0) {
        val.coerce_to_string(activation)?
    } else {
        return Ok(Value::Undefined);
    };
    let socket_id = this
        .as_xml_socket()
        .expect("only available on XMLSocket; qed;")
        .id;
    activation
        .context
        .navigator
        .xmlsocket_send(&socket_id, s.as_bytes().to_vec());
    Ok(Value::Undefined)
}

pub fn get_on_xml<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onXML GET");
    this.get("_onXML", activation)
}

pub fn set_on_xml<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onXML SET not implemented");
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    this.set("_onXML", callback.into(), activation)?;
    Ok(Value::Undefined)
}

pub fn get_on_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onData GET");
    this.get("_onData", activation)
}

pub fn set_on_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onData SET");
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    this.set("_onData", callback.into(), activation)?;
    Ok(Value::Undefined)
}

pub fn get_on_connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onConnect GET");
    this.get("_onConnect", activation)
}

pub fn set_on_connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onConnect SET");
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    this.set("_onConnect", callback.into(), activation)?;
    Ok(Value::Undefined)
}

pub fn get_on_close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onClose GET");
    this.get("_onClose", activation)
}

pub fn set_on_close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.onClose SET");
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    this.set("_onClose", callback.into(), activation)?;
    Ok(Value::Undefined)
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.connect()");
    let host: String = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .parse()
        .unwrap_or_default();
    let port: f64 = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)
        .unwrap_or_default();
    let socket_id = this
        .as_xml_socket()
        .expect("only available on XMLSocket; qed;")
        .id;
    activation
        .context
        .xml_socket
        .sockets
        .insert(socket_id, this);
    activation.context.navigator.xmlsocket_connect(
        socket_id,
        ConnectOptions {
            host,
            port: port as u16,
        },
    );
    /* NOTE: the doc is stating that connect()
       should return true if a connection could be opened,
       but the ultimate source of truth is the onConnect callback
       http://demo.ligams.free.fr/AS2LR/XMLSocket.html
    */
    Ok(true.into())
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "XMLSocket.close() not implemented");
    Ok(Value::Undefined)
}

pub fn create_xml_socket_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    xml_socket_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_socket = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        Some(fn_proto),
        xml_socket_proto,
    );
    let object = xml_socket.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    xml_socket
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_socket = XmlSocket::empty_socket(gc_context, Some(proto), 0);
    let object = xml_socket.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    xml_socket
}

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}
