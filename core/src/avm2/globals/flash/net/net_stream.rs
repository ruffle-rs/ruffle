use crate::avm2::{Activation, Error, Object, TObject, Value};

pub use crate::avm2::object::netstream_allocator as net_stream_allocator;

pub fn get_bytes_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(ns) = this.and_then(|o| o.as_netstream()) {
        return Ok(ns.bytes_loaded().into());
    }

    Ok(Value::Undefined)
}

pub fn get_bytes_total<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(ns) = this.and_then(|o| o.as_netstream()) {
        return Ok(ns.bytes_total().into());
    }

    Ok(Value::Undefined)
}
