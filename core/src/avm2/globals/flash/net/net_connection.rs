use crate::avm2::error::make_error_2126;
pub use crate::avm2::object::net_connection_allocator;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::net_connection::NetConnections;
use crate::string::AvmString;
use crate::{
    avm2::{Activation, Error, Object, Value},
    avm2_stub_method,
};
use ruffle_wstr::WStr;

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Value::Null = args[0] {
        NetConnections::connect_to_local(&mut activation.context, connection);
        return Ok(Value::Undefined);
    }

    let url = args.get_string(activation, 0)?;
    if url.starts_with(WStr::from_units(b"http://"))
        || url.starts_with(WStr::from_units(b"https://"))
    {
        // HTTP(S) is for Flash Remoting, which is just POST requests to the URL.
        NetConnections::connect_to_flash_remoting(
            &mut activation.context,
            connection,
            url.to_string(),
        );
    } else {
        avm2_stub_method!(
            activation,
            "flash.net.NetConnection",
            "connect",
            "with non-null, non-http command"
        );
    }

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .as_net_connection()
        .expect("Must be NetConnection object");
    if let Some(previous_handle) = connection.set_handle(None) {
        NetConnections::close(&mut activation.context, previous_handle);
    }

    Ok(Value::Undefined)
}

pub fn get_connected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(handle) = this.handle() {
        return Ok(activation
            .context
            .net_connections
            .is_connected(handle)
            .into());
    }

    Ok(false.into())
}

pub fn get_connected_proxy_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this.handle().and_then(|handle| {
        activation
            .context
            .net_connections
            .get_connected_proxy_type(handle)
    }) {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_far_id<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_far_id(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_far_nonce<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_far_nonce(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_near_id<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_near_id(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_near_nonce<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_near_nonce(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_protocol<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_protocol(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}

pub fn get_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.get_uri(handle))
    {
        return Ok(AvmString::new_utf8(activation.context.gc_context, result).into());
    }

    Ok(Value::Null)
}

pub fn get_using_tls<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Some(result) = this
        .handle()
        .and_then(|handle| activation.context.net_connections.is_using_tls(handle))
    {
        return Ok(result.into());
    }

    Err(make_error_2126(activation))
}
