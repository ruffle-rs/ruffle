pub use crate::avm2::object::socket_allocator;
use crate::{
    avm2::{Activation, Error, Object, TObject, Value},
    context::UpdateContext,
};

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let socket = match this.as_socket() {
        Some(socket) => socket,
        None => return Ok(Value::Undefined),
    };

    let host = match args.get(0) {
        Some(host) => host,
        // FIXME: What error should we use here?
        None => todo!(),
    }
    .coerce_to_string(activation)?;

    // FIXME: How do you get u16? there is no coerce_to_u16 method.
    let port = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_u32(activation)? as u16;

    let UpdateContext {
        sockets, navigator, ..
    } = &mut activation.context;

    if let Some(handle) = sockets.connect(*navigator, this, &host.to_utf8_lossy(), port) {
        if let Some(previous_handle) = socket.set_handle(handle) {
            // As written in the AS3 docs, we are supposed to close the existing connection,
            // when a new one is created.
            sockets.close(previous_handle);
        }
    };

    // FIXME: Are we supposed to throw and IOError when a connection fails?

    Ok(Value::Undefined)
}

pub fn get_connected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let socket = match this.as_socket() {
        Some(socket) => socket,
        None => return Ok(Value::Undefined),
    };

    let UpdateContext { sockets, .. } = &mut activation.context;

    let handle = match socket.get_handle() {
        Some(handle) => handle,
        None => return Ok(Value::Bool(false)),
    };

    let is_connected = sockets.is_connected(handle).unwrap_or(false);

    Ok(Value::Bool(is_connected))
}
