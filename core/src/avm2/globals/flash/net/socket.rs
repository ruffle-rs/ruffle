use crate::avm2::bytearray::Endian;
use crate::avm2::error::make_error_2008;
pub use crate::avm2::object::socket_allocator;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::context::UpdateContext;

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

pub fn get_bytes_available<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket.read_buffer().len().into());
    }

    Ok(Value::Undefined)
}

pub fn get_endian<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(match socket.endian() {
            Endian::Big => "bigEndian".into(),
            Endian::Little => "littleEndian".into(),
        });
    }

    Ok(Value::Undefined)
}

pub fn set_endian<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let endian = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;
        if &endian == b"bigEndian" {
            socket.set_endian(Endian::Big);
        } else if &endian == b"littleEndian" {
            socket.set_endian(Endian::Little);
        } else {
            return Err(make_error_2008(activation, "value"));
        }
    }

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

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let UpdateContext { sockets, .. } = &mut activation.context;

        // FIXME: Throw correct IoError
        let handle = socket.get_handle().unwrap();
        let data = socket.drain_write_buf();

        sockets.send(handle, data)
    }

    Ok(Value::Undefined)
}

pub fn write_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let byte = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        socket.write_bytes(&[byte as u8]);
    }

    Ok(Value::Undefined)
}

pub fn write_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let bytearray = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let offset = args
            .get(1)
            .unwrap_or(&Value::Integer(0))
            .coerce_to_u32(activation)? as usize;
        let length = args
            .get(2)
            .unwrap_or(&Value::Integer(0))
            .coerce_to_u32(activation)? as usize;

        let ba_read = bytearray
            .as_bytearray()
            .ok_or("ArgumentError: Parameter must be a bytearray")?;

        let to_write = ba_read
            .read_at(
                // If length is 0, lets read the remaining bytes of ByteArray from the supplied offset
                if length != 0 {
                    length
                } else {
                    ba_read.len().saturating_sub(offset)
                },
                offset,
            )
            .map_err(|e| e.to_avm(activation))?;

        socket.write_bytes(to_write);
    }

    Ok(Value::Undefined)
}
