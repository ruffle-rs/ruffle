use crate::avm1::function::FunctionObject;
use crate::avm1::object::{NativeObject, Object};
use crate::avm1::property_decl::define_properties_on;
use crate::avm1::{property_decl::Declaration, ScriptObject};
use crate::avm1::{Activation, Error, Executable, ExecutionReason, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;
use crate::socket::SocketHandle;
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, Gc};
use std::cell::{Cell, RefCell, RefMut};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct XmlSocketData {
    handle: Cell<Option<SocketHandle>>,
    /// Connection timeout in milliseconds.
    timeout: Cell<u32>,
    read_buffer: RefCell<Vec<u8>>,
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct XmlSocket<'gc>(Gc<'gc, XmlSocketData>);

impl<'gc> XmlSocket<'gc> {
    pub fn handle(&self) -> Option<SocketHandle> {
        self.0.handle.get()
    }

    pub fn set_handle(&self, handle: SocketHandle) -> Option<SocketHandle> {
        self.0.handle.replace(Some(handle))
    }

    pub fn timeout(&self) -> u32 {
        self.0.timeout.get()
    }

    pub fn set_timeout(&self, new_timeout: u32) {
        // FIXME: Check if flash player clamps this to 250 milliseconds like AS3 sockets.
        self.0.timeout.set(new_timeout);
    }

    pub fn read_buffer(&self) -> RefMut<'_, Vec<u8>> {
        self.0.read_buffer.borrow_mut()
    }

    pub fn cast(value: Value<'gc>) -> Option<Self> {
        if let Value::Object(object) = value {
            if let NativeObject::XmlSocket(xml_socket) = object.native() {
                return Some(xml_socket);
            }
        }
        None
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "timeout" => property(get_timeout, set_timeout);
    "close" => method(close);
    "connect" => method(connect);
    "send" => method(send);
    "onConnect" => method(on_connect; DONT_ENUM | DONT_DELETE);
    "onClose" => method(on_close; DONT_ENUM | DONT_DELETE);
    "onData" => method(on_data; DONT_ENUM | DONT_DELETE);
    "onXML" => method(on_xml; DONT_ENUM | DONT_DELETE);
};

fn get_timeout<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xml_socket) = XmlSocket::cast(this.into()) {
        Ok(xml_socket.timeout().into())
    } else {
        Ok(Value::Undefined)
    }
}

fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xml_socket) = XmlSocket::cast(this.into()) {
        let timeout = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

        xml_socket.set_timeout(timeout);
    }

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xml_socket) = XmlSocket::cast(this.into()) {
        if let Some(handle) = xml_socket.handle() {
            activation.context.sockets.close(handle)
        }
    }

    Ok(Value::Undefined)
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if XmlSocket::cast(this.into()).is_some() {
        let host = args
            .get(0)
            .copied()
            .unwrap_or_else(|| {
                let movie = activation.base_clip().movie();

                if let Ok(url) = url::Url::parse(movie.url()) {
                    if url.scheme() == "file" {
                        "localhost".into()
                    } else if let Some(domain) = url.domain() {
                        AvmString::new_utf8(activation.context.gc_context, domain).into()
                    } else {
                        // no domain?
                        "localhost".into()
                    }
                } else {
                    Value::Undefined
                }
            })
            .coerce_to_string(activation)?;
        let port = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u16(activation)?;

        let UpdateContext {
            sockets, navigator, ..
        } = activation.context;

        sockets.connect_avm1(*navigator, this, host.to_utf8_lossy().into_owned(), port);

        // NOTE: At this point we do not know if the connection will succeed
        //       because connecting is an asynchronous process, so we just return true.
        return Ok(true.into());
    }

    Ok(Value::Undefined)
}

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xml_socket) = XmlSocket::cast(this.into()) {
        if let Some(handle) = xml_socket.handle() {
            let mut data = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?
                .to_string()
                .into_bytes();

            // The string needs to end with a null byte.
            data.push(0);

            activation.context.sockets.send(handle, data);
        }
    }

    Ok(Value::Undefined)
}

fn on_connect<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op by default
    Ok(Value::Undefined)
}

fn on_close<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op by default
    Ok(Value::Undefined)
}

fn on_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
        tracing::warn!("default XMLSocket.onData() received invalid XML; message ignored");
    }

    Ok(Value::Undefined)
}

fn on_xml<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op by default
    Ok(Value::Undefined)
}

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_socket = XmlSocket(Gc::new(
        activation.gc(),
        XmlSocketData {
            handle: Cell::new(None),
            // Default timeout is 20_000 milliseconds (20 seconds)
            timeout: Cell::new(20000),
            read_buffer: RefCell::new(Vec::new()),
        },
    ));

    this.set_native(activation.gc(), NativeObject::XmlSocket(xml_socket));

    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_socket_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, xml_socket_proto, fn_proto);
    xml_socket_proto.into()
}

pub fn create_class<'gc>(
    context: &mut StringContext<'gc>,
    xml_socket_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        xml_socket_proto,
    )
}
