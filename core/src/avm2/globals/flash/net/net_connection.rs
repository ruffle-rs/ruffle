pub use crate::avm2::object::net_connection_allocator;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::net_connection::NetConnections;
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
