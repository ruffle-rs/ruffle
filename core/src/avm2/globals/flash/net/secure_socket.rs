use crate::avm2::error::make_error_2003;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, Error, Value};
use crate::context::UpdateContext;

/// Implements `SecureSocket.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let socket = match this.as_socket() {
        Some(socket) => socket,
        None => return Ok(Value::Undefined),
    };

    let host = args.get_string(activation, 0);
    let port = args.get_i32(1);
    if !(1..=65535).contains(&port) {
        return Err(make_error_2003(activation));
    }
    let port = port as u16;

    let UpdateContext {
        sockets, navigator, ..
    } = activation.context;

    sockets.connect_avm2_secure(*navigator, socket, host.to_utf8_lossy().into_owned(), port);

    Ok(Value::Undefined)
}

/// Implements `SecureSocket.isSupported`
pub fn get_is_supported<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // SecureSocket is supported in Ruffle (we use native TLS).
    Ok(true.into())
}

/// Implements `SecureSocket.serverCertificateStatus`
pub fn get_server_certificate_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(socket) = this.as_socket() {
        let status = socket.certificate_status();
        return Ok(AvmString::new_utf8(activation.gc(), &status).into());
    }

    Ok(Value::Undefined)
}
