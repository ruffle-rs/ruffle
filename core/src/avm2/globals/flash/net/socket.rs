use std::rc::Rc;

use crate::avm2::bytearray::{Endian, ObjectEncoding};
use crate::avm2::error::{io_error, make_error_2008, security_error};
pub use crate::avm2::object::socket_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::context::UpdateContext;
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf3::read::AMF3Decoder;
use flash_lso::types::{AMFVersion, Element};

macro_rules! assert_socket_open {
    ($activation:expr, $socket:expr) => {
        let handle = $socket
            .handle()
            .ok_or_else(|| invalid_socket_error($activation))?;

        if !$activation.context.sockets.is_connected(handle) {
            return Err(invalid_socket_error($activation));
        }
    };
}

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
    } = activation.context;

    sockets.connect_avm2(*navigator, socket, host.to_utf8_lossy().into_owned(), port);

    Ok(Value::Undefined)
}

pub fn get_timeout<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok(socket.timeout().into());
    }

    Ok(Value::Undefined)
}

pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let new_timeout = args.get_u32(activation, 0)?;
        socket.set_timeout(new_timeout)
    }

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        // We throw an IOError when socket is not open.
        let handle = socket.handle().ok_or(invalid_socket_error(activation))?;

        if !activation.context.sockets.is_connected(handle) {
            return Err(invalid_socket_error(activation));
        }

        let UpdateContext { sockets, .. } = activation.context;

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

    let UpdateContext { sockets, .. } = activation.context;

    let handle = match socket.handle() {
        Some(handle) => handle,
        None => return Ok(Value::Bool(false)),
    };

    Ok(Value::Bool(sockets.is_connected(handle)))
}

pub fn get_object_encoding<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        return Ok((socket.object_encoding() as u8).into());
    }

    Ok(Value::Undefined)
}

pub fn set_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let new_encoding = args.get_u32(activation, 0)?;
        match new_encoding {
            0 => socket.set_object_encoding(ObjectEncoding::Amf0),
            3 => socket.set_object_encoding(ObjectEncoding::Amf3),
            _ => return Err(make_error_2008(activation, "objectEncoding")),
        }
    }

    Ok(Value::Undefined)
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        let handle = socket.handle().ok_or(invalid_socket_error(activation))?;
        if !activation.context.sockets.is_connected(handle) {
            return Err(invalid_socket_error(activation));
        }

        let UpdateContext { sockets, .. } = activation.context;

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
            .as_bytearray_mut()
            .expect("Parameter must be a bytearray!");

        ba_write
            .write_at(&to_write, offset)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn read_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

        return Ok(socket.read_int().map_err(|e| e.to_avm(activation))?.into());
    }

    Ok(Value::Undefined)
}

pub fn read_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

        let len = args.get_u32(activation, 0)?;
        let charset_label = args.get_string(activation, 1)?;
        let mut bytes = &*socket
            .read_bytes(len as usize)
            .map_err(|e| e.to_avm(activation))?;

        // Flash cuts off the string at the first null byte (after checking that
        // the original length fits in the ByteArray)
        if let Some(null) = bytes.iter().position(|b| *b == b'\0') {
            bytes = &bytes[..null];
        }

        let encoder =
            Encoding::for_label(charset_label.to_utf8_lossy().as_bytes()).unwrap_or(UTF_8);
        let (decoded_str, _, _) = encoder.decode(bytes);
        return Ok(AvmString::new_utf8(activation.gc(), decoded_str).into());
    }

    Ok(Value::Undefined)
}

pub fn read_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

        let mut bytes = socket.read_buffer();

        let (bytes_left, value) = match socket.object_encoding() {
            ObjectEncoding::Amf0 => {
                let mut decoder = AMF0Decoder::default();
                let (extra, amf) = decoder
                    .parse_single_element(&bytes)
                    .map_err(|_| "Error: Invalid object")?;
                (
                    extra.len(),
                    crate::avm2::amf::deserialize_value(activation, &amf)?,
                )
            }
            ObjectEncoding::Amf3 => {
                let mut decoder = AMF3Decoder::default();
                let (extra, amf) = decoder
                    .parse_single_element(&bytes)
                    .map_err(|_| "Error: Invalid object")?;
                (
                    extra.len(),
                    crate::avm2::amf::deserialize_value(activation, &amf)?,
                )
            }
        };

        let len = bytes.len();
        let _ = bytes.drain(..(len - bytes_left));
        return Ok(value);
    }

    Ok(Value::Undefined)
}

pub fn read_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

        let num = args.get_i32(activation, 0)?;
        socket.write_int(num);
    }

    Ok(Value::Undefined)
}

pub fn write_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

        let string = args.get_string(activation, 0)?;
        let charset_label = args.get_string(activation, 1)?;

        let encoder =
            Encoding::for_label(charset_label.to_utf8_lossy().as_bytes()).unwrap_or(UTF_8);
        let utf8 = string.to_utf8_lossy();
        let (encoded_bytes, _, _) = encoder.encode(&utf8);
        socket.write_bytes(&encoded_bytes);
    }

    Ok(Value::Undefined)
}

pub fn write_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

        let obj = args.get_value(0);
        let amf_version = match socket.object_encoding() {
            ObjectEncoding::Amf0 => AMFVersion::AMF0,
            ObjectEncoding::Amf3 => AMFVersion::AMF3,
        };

        let amf = crate::avm2::amf::serialize_value(
            activation,
            obj,
            amf_version,
            &mut Default::default(),
        )
        .unwrap_or(flash_lso::types::Value::Undefined);

        let element = Element::new("", Rc::new(amf));
        let mut lso = flash_lso::types::Lso::new(vec![element], "", amf_version);
        let bytes =
            flash_lso::write::write_to_bytes(&mut lso).map_err(|_| "Failed to serialize object")?;
        // This is kind of hacky: We need to strip out the header and any padding so that we only write
        // the value. In the future, there should be a method to do this in the flash_lso crate.
        let element_padding = match amf_version {
            AMFVersion::AMF0 => 8,
            AMFVersion::AMF3 => 7,
        };
        socket.write_bytes(
            &bytes[flash_lso::write::header_length(&lso.header) + element_padding..bytes.len() - 1],
        );
    }

    Ok(Value::Undefined)
}

pub fn write_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(socket) = this.as_socket() {
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
        assert_socket_open!(activation, socket);

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
