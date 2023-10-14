use crate::{
    avm2::{Activation, Avm2, Error, EventObject, Object, Value},
    avm2_stub_method,
};

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Null = args[0] {
        let event = EventObject::net_status_event(
            activation,
            "netStatus",
            vec![
                ("code", "NetConnection.Connect.Success"),
                ("level", "status"),
            ],
        );
        Avm2::dispatch_event(&mut activation.context, event, this);
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
