pub use crate::avm2::object::net_connection_allocator;
use crate::avm2::object::TObject;
use crate::net_connection::NetConnections;
use crate::{
    avm2::{Activation, Error, Object, Value},
    avm2_stub_method,
};

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
    avm2_stub_method!(
        activation,
        "flash.net.NetConnection",
        "connect",
        "with non-null command"
    );
    Ok(Value::Undefined)
}
