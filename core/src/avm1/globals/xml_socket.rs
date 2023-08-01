use crate::avm1::function::FunctionObject;
use crate::avm1::object::{NativeObject, Object};
use crate::avm1::property_decl::define_properties_on;
use crate::avm1::{property_decl::Declaration, ScriptObject};
use crate::avm1::{Activation, Error, Executable, TObject, Value};
use crate::context::GcContext;
use crate::socket::SocketHandle;
use gc_arena::{Collect, GcCell, Mutation};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct XmlSocketData {
    handle: Option<SocketHandle>,
    /// Connection timeout in milliseconds.
    timeout: u32,
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct XmlSocket<'gc>(GcCell<'gc, XmlSocketData>);

impl<'gc> XmlSocket<'gc> {
    pub fn handle(&self) -> Option<SocketHandle> {
        self.0.read().handle
    }

    pub fn set_handle(
        &self,
        gc_context: &Mutation<'gc>,
        handle: SocketHandle,
    ) -> Option<SocketHandle> {
        std::mem::replace(&mut self.0.write(gc_context).handle, Some(handle))
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_socket = XmlSocket(GcCell::new(
        activation.gc(),
        XmlSocketData {
            handle: None,
            /// Default timeout is 20_000 milliseconds (20 seconds)
            timeout: 20000,
        },
    ));

    this.set_native(activation.gc(), NativeObject::XmlSocket(xml_socket));

    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_socket_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, xml_socket_proto, fn_proto);
    xml_socket_proto.into()
}

pub fn create_class<'gc>(
    context: &mut GcContext<'_, 'gc>,
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
