use crate::avm2::bytearray::Endian;
use crate::avm2::error::{io_error, make_error_2008, security_error};
pub use crate::avm2::object::socket_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::string::AvmString;
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

    let host = args.get_string(activation, 0)?;
    let port = args.get_u32(activation, 1)?;
    let port: u16 = port
        .try_into()
        .map_err(|_| invalid_port_number(activation))?;

    let UpdateContext {
        sockets, navigator, ..
    } = &mut activation.context;

    sockets.connect(*navigator, socket, host.to_utf8_lossy().into_owned(), port);

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        // We throw an IOError when socket is not open.
        let handle = socket
            .get_handle()
            .ok_or(invalid_socket_error(activation))?;

        let UpdateContext { sockets, .. } = &mut activation.context;

        sockets.close(handle)
    }

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
        let endian = args.get_string(activation, 0)?;
        if &endian == b"bigEndian" {
            socket.set_endian(Endian::Big);
        } else if &endian == b"littleEndian" {
            socket.set_endian(Endian::Little);
        } else {
            return Err(make_error_2008(activation, "endian"));
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

    Ok(Value::Bool(sockets.is_connected(handle)))
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let handle = socket
            .get_handle()
            .ok_or(invalid_socket_error(activation))?;
        let UpdateContext { sockets, .. } = &mut activation.context;

        let mut buffer = socket.write_buffer();
        let len = buffer.len();
        let data = buffer.drain(..len).collect::<Vec<_>>();

        sockets.send(handle, data)
    }

    Ok(Value::Undefined)
}

pub fn read_boolean<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_boolean()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket.read_byte().map_err(|e| e.to_avm(activation))?.into());
    }

    Ok(Value::Undefined)
}

pub fn read_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let bytearray = args.get_object(activation, 0, "bytes")?;
        let offset = args.get_u32(activation, 1)? as usize;
        let length = args.get_u32(activation, 2)? as usize;

        let to_write = socket
            .read_bytes(if length != 0 {
                length
            } else {
                socket.read_buffer().len()
            })
            .map_err(|e| e.to_avm(activation))?;

        let mut ba_write = bytearray
            .as_bytearray_mut(activation.gc())
            .expect("Parameter must be a bytearray!");

        ba_write.write_at(&to_write, offset)?;
    }

    Ok(Value::Undefined)
}

pub fn read_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_double()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_float()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket.read_int().map_err(|e| e.to_avm(activation))?.into());
    }

    Ok(Value::Undefined)
}

pub fn read_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_short()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_unsigned_byte()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_unsigned_int()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket
            .read_unsigned_short()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_utf<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(AvmString::new_utf8_bytes(
            activation.gc(),
            &socket.read_utf().map_err(|e| e.to_avm(activation))?,
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn read_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let length = args.get_u32(activation, 0)?;

        return Ok(AvmString::new_utf8_bytes(
            activation.gc(),
            &socket
                .read_utf_bytes(length as usize)
                .map_err(|e| e.to_avm(activation))?,
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn write_boolean<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let byte = args.get_bool(0);
        socket.write_boolean(byte);
    }

    Ok(Value::Undefined)
}

pub fn write_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let byte = args.get_u32(activation, 0)?;
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
        let bytearray = args.get_object(activation, 0, "bytes")?;
        let offset = args.get_u32(activation, 1)? as usize;
        let length = args.get_u32(activation, 2)? as usize;

        let ba_read = bytearray
            .as_bytearray()
            .expect("Parameter must be a bytearray!");

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

pub fn write_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let num = args.get_f64(activation, 0)?;
        socket.write_double(num);
    }

    Ok(Value::Undefined)
}

pub fn write_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let num = args.get_f64(activation, 0)?;
        socket.write_float(num as f32);
    }

    Ok(Value::Undefined)
}

pub fn write_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let num = args.get_i32(activation, 0)?;
        socket.write_int(num);
    }

    Ok(Value::Undefined)
}

pub fn write_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let num = args.get_i32(activation, 0)?;
        socket.write_short(num as i16);
    }

    Ok(Value::Undefined)
}

pub fn write_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let num = args.get_u32(activation, 0)?;
        socket.write_unsigned_int(num);
    }

    Ok(Value::Undefined)
}

pub fn write_utf<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let string = args.get_string(activation, 0)?;

        socket.write_utf(&string.to_utf8_lossy())?;
    }

    Ok(Value::Undefined)
}

pub fn write_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let string = args.get_string(activation, 0)?;

        socket.write_bytes(string.to_utf8_lossy().as_bytes());
    }

    Ok(Value::Undefined)
}

fn invalid_socket_error<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    match io_error(
        activation,
        "Error #2002: Operation attempted on invalid socket.",
        2002,
    ) {
        Ok(err) => Error::AvmError(err),
        Err(e) => e,
    }
}

fn invalid_port_number<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    match security_error(
        activation,
        "Error #2003: Invalid socket port number specified.",
        2003,
    ) {
        Ok(err) => Error::AvmError(err),
        Err(e) => e,
    }
}
