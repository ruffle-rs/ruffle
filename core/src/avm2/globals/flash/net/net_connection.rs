use crate::avm2::amf::serialize_value;
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
use flash_lso::packet::Header;
use flash_lso::types::AMFVersion;
use flash_lso::types::Value as AMFValue;
use fnv::FnvHashMap;
use ruffle_wstr::WStr;
use std::rc::Rc;

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    if let Value::Null = args[0] {
        NetConnections::connect_to_local(activation.context, connection);
        return Ok(Value::Undefined);
    }

    let url = args.get_string(activation, 0)?;
    if url.starts_with(WStr::from_units(b"http://"))
        || url.starts_with(WStr::from_units(b"https://"))
    {
        // HTTP(S) is for Flash Remoting, which is just POST requests to the URL.
        NetConnections::connect_to_flash_remoting(activation.context, connection, url.to_string());
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
        NetConnections::close(activation.context, previous_handle, true);
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

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    let command = args.get_string(activation, 0)?;
    let responder = args
        .try_get_object(activation, 1)
        .and_then(|o| o.as_responder());
    let mut arguments = Vec::new();

    let mut object_table = FnvHashMap::default();
    for arg in &args[2..] {
        if let Some(value) = serialize_value(activation, *arg, AMFVersion::AMF0, &mut object_table)
        {
            arguments.push(Rc::new(value));
        }
    }

    if let Some(handle) = connection.handle() {
        if let Some(responder) = responder {
            NetConnections::send_avm2(
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

        return Ok(Value::Undefined);
    }

    Err(make_error_2126(activation))
}

pub fn add_header<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .as_net_connection()
        .expect("Must be NetConnection object");

    // [NA] The documentation says that the header persists for the duration of this object.
    // However, this doesn't seem to be true - if you set a header and then open a connection,
    // the header is lost.
    // Therefore, we'll only store them on an active connection object - and lose them otherwise.

    // [NA] Another thing the docs have wrong, it says that you can remove a header by just calling
    // `addHeader(name)` - but this is clearly false. It instead replaces the value of the header
    // with a null value, sending that over the wire.

    let name = args.get_string(activation, 0)?;
    let must_understand = args.get_bool(1);
    // FIXME - do we re-use the same object reference table for all headers?
    let value = serialize_value(
        activation,
        args[2],
        AMFVersion::AMF0,
        &mut Default::default(),
    )
    .unwrap_or(AMFValue::Null);

    if let Some(handle) = connection.handle() {
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
