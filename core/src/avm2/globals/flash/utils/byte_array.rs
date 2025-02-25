use std::rc::Rc;

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::{Endian, ObjectEncoding};
use crate::avm2::error::make_error_2008;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf3::read::AMF3Decoder;
use flash_lso::types::{AMFVersion, Element};
use ruffle_macros::istr;
use ruffle_wstr::WString;

pub use crate::avm2::object::byte_array_allocator;

/// Writes a single byte to the bytearray
pub fn write_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let byte = args.get_i32(activation, 0)?;
        bytearray
            .write_bytes(&[byte as u8])
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

/// Writes multiple bytes to the bytearray from another bytearray
pub fn write_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let bytearray = args.get_object(activation, 0, "bytes")?;
    let offset = args.get_u32(activation, 1)? as usize;
    let length = args.get_u32(activation, 2)? as usize;

    if !Object::ptr_eq(this, bytearray) {
        // The ByteArray we are reading from is different than the ByteArray we are writing to,
        // so we are allowed to borrow both at the same time without worrying about a panic

        let ba_read = bytearray
            .as_bytearray()
            .expect("Parameter must be a bytearray");
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

        if let Some(mut bytearray) = this.as_bytearray_mut() {
            bytearray
                .write_bytes(to_write)
                .map_err(|e| e.to_avm(activation))?;
        }
    } else if let Some(mut bytearray) = this.as_bytearray_mut() {
        // The ByteArray we are reading from is the same as the ByteArray we are writing to,
        // so we only need to borrow once, and we can use `write_bytes_within` to write bytes from our own ByteArray
        let amnt = if length != 0 {
            length
        } else {
            bytearray.len().saturating_sub(offset)
        };
        bytearray
            .write_bytes_within(offset, amnt)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

// Reads the bytes from the current bytearray into another bytearray
pub fn read_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let bytearray = args.get_object(activation, 0, "bytes")?;
    let offset = args.get_u32(activation, 1)? as usize;
    let length = args.get_u32(activation, 2)? as usize;

    if !Object::ptr_eq(this, bytearray) {
        if let Some(bytearray_read) = this.as_bytearray() {
            let to_write = bytearray_read
                .read_bytes(
                    // If length is 0, lets read the remaining bytes of ByteArray
                    if length != 0 {
                        length
                    } else {
                        bytearray_read.bytes_available()
                    },
                )
                .map_err(|e| e.to_avm(activation))?;

            let mut ba_write = bytearray
                .as_bytearray_mut()
                .expect("Parameter must be a bytearray");

            ba_write
                .write_at(to_write, offset)
                .map_err(|e| e.to_avm(activation))?;
        }
    } else if let Some(mut bytearray) = this.as_bytearray_mut() {
        let amnt = if length != 0 {
            length
        } else {
            bytearray.bytes_available()
        };
        let pos = bytearray.position();
        bytearray
            .write_at_within(pos, amnt, offset)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}
pub fn write_utf<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let utf_string = args.get_string(activation, 0)?;

        // NOTE: there is a bug on old Flash Player (e.g. v11.3); if the string to
        // write ends with an unpaired high surrogate, the routine bails out and nothing
        // is written.
        // The bug is fixed on newer FP versions (e.g. v32), but the fix isn't SWF-version-gated.
        bytearray
            .write_utf(&utf_string.to_utf8_lossy())
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn read_utf<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(AvmString::new_utf8_bytes(
            activation.gc(),
            bytearray.read_utf().map_err(|e| e.to_avm(activation))?,
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn strip_bom<'gc>(activation: &mut Activation<'_, 'gc>, mut bytes: &[u8]) -> AvmString<'gc> {
    // UTF-8 BOM
    if let Some(without_bom) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        bytes = without_bom;
    // Little-endian UTF-16 BOM
    } else if let Some(without_bom) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let utf16_bytes: Vec<_> = without_bom
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        return AvmString::new(activation.gc(), WString::from_buf(utf16_bytes));
    // Big-endian UTF-16 BOM
    } else if let Some(without_bom) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let utf16_bytes: Vec<_> = without_bom
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect();
        return AvmString::new(activation.gc(), WString::from_buf(utf16_bytes));
    }

    AvmString::new_utf8_bytes(activation.gc(), bytes)
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(strip_bom(activation, bytearray.bytes()).into());
    }

    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        bytearray.clear();
        bytearray.shrink_to_fit();
    }

    Ok(Value::Undefined)
}

pub fn get_position<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray.position().into());
    }

    Ok(Value::Undefined)
}

pub fn set_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        let num = args.get_u32(activation, 0)?;
        bytearray.set_position(num as usize);
    }

    Ok(Value::Undefined)
}

pub fn get_bytes_available<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray.bytes_available().into());
    }

    Ok(Value::Undefined)
}

pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray.len().into());
    }

    Ok(Value::Undefined)
}

pub fn set_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let len = args.get_u32(activation, 0)? as usize;
        bytearray.set_length(len);
    }

    Ok(Value::Undefined)
}

pub fn get_endian<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(match bytearray.endian() {
            Endian::Big => istr!("bigEndian").into(),
            Endian::Little => istr!("littleEndian").into(),
        });
    }

    Ok(Value::Undefined)
}

pub fn set_endian<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let endian = args.get_string(activation, 0)?;
        if &endian == b"bigEndian" {
            bytearray.set_endian(Endian::Big);
        } else if &endian == b"littleEndian" {
            bytearray.set_endian(Endian::Little);
        } else {
            return Err(make_error_2008(activation, "endian"));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_short()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_unsigned_short()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_double()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_float()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_int()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_unsigned_int()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_boolean<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_boolean()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_byte()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn read_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        let len = args.get_u32(activation, 0)?;
        return Ok(AvmString::new_utf8(
            activation.gc(),
            String::from_utf8_lossy(
                bytearray
                    .read_utf_bytes(len as usize)
                    .map_err(|e| e.to_avm(activation))?,
            ),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok(bytearray
            .read_unsigned_byte()
            .map_err(|e| e.to_avm(activation))?
            .into());
    }

    Ok(Value::Undefined)
}

pub fn write_float<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_f64(activation, 0)?;
        bytearray
            .write_float(num as f32)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_f64(activation, 0)?;
        bytearray
            .write_double(num)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_boolean<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_bool(0);
        bytearray
            .write_boolean(num)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_i32(activation, 0)?;
        bytearray.write_int(num).map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_u32(activation, 0)?;
        bytearray
            .write_unsigned_int(num)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_short<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let num = args.get_i32(activation, 0)?;
        bytearray
            .write_short(num as i16)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn write_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let string = args.get_string(activation, 0)?;
        let charset_label = args.get_string(activation, 1)?;
        let encoder =
            Encoding::for_label(charset_label.to_utf8_lossy().as_bytes()).unwrap_or(UTF_8);
        let utf8 = string.to_utf8_lossy();
        let (encoded_bytes, _, _) = encoder.encode(&utf8);
        bytearray
            .write_bytes(&encoded_bytes)
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn read_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        let len = args.get_u32(activation, 0)?;
        let charset_label = args.get_string(activation, 1)?;
        let mut bytes = bytearray
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

pub fn write_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let string = args.get_string(activation, 0)?;
        bytearray
            .write_bytes(string.to_utf8_lossy().as_bytes())
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn compress<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let algorithm = args.get_string(activation, 0)?;
        let algorithm = match algorithm.parse() {
            Ok(algorithm) => algorithm,
            Err(_) => {
                return Err(Error::AvmError(crate::avm2::error::io_error(
                    activation,
                    "Error #2058: There was an error decompressing the data.",
                    2058,
                )?))
            }
        };
        let buffer = bytearray.compress(algorithm);
        bytearray.clear();
        bytearray
            .write_bytes(&buffer)
            .map_err(|e| e.to_avm(activation))?;
        bytearray.set_position(bytearray.len());
    }

    Ok(Value::Undefined)
}

pub fn uncompress<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let algorithm = args.get_string(activation, 0)?;
        let algorithm = match algorithm.parse() {
            Ok(algorithm) => algorithm,
            Err(_) => {
                return Err(Error::AvmError(crate::avm2::error::io_error(
                    activation,
                    "Error #2058: There was an error decompressing the data.",
                    2058,
                )?))
            }
        };
        let buffer = match bytearray.decompress(algorithm) {
            Some(buffer) => buffer,
            None => {
                return Err(Error::AvmError(crate::avm2::error::io_error(
                    activation,
                    "Error #2058: There was an error decompressing the data.",
                    2058,
                )?))
            }
        };
        bytearray.clear();
        bytearray
            .write_bytes(&buffer)
            .map_err(|e| e.to_avm(activation))?;
        bytearray.set_position(0);
    }

    Ok(Value::Undefined)
}

pub fn read_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        let bytes = bytearray
            .read_at(bytearray.bytes_available(), bytearray.position())
            .map_err(|e| e.to_avm(activation))?;

        let (bytes_left, value) = match bytearray.object_encoding() {
            ObjectEncoding::Amf0 => {
                let mut decoder = AMF0Decoder::default();
                let (extra, amf) = decoder
                    .parse_single_element(bytes)
                    .map_err(|_| "Error: Invalid object")?;
                (
                    extra.len(),
                    crate::avm2::amf::deserialize_value(activation, &amf)?,
                )
            }
            ObjectEncoding::Amf3 => {
                let mut decoder = AMF3Decoder::default();
                let (extra, amf) = decoder
                    .parse_single_element(bytes)
                    .map_err(|_| "Error: Invalid object")?;
                (
                    extra.len(),
                    crate::avm2::amf::deserialize_value(activation, &amf)?,
                )
            }
        };

        bytearray.set_position(bytearray.len() - bytes_left);
        return Ok(value);
    }

    Ok(Value::Undefined)
}

pub fn write_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let obj = args.get_value(0);
        let amf_version = match bytearray.object_encoding() {
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
        bytearray
            .write_bytes(
                &bytes[flash_lso::write::header_length(&lso.header) + element_padding
                    ..bytes.len() - 1],
            )
            .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

pub fn get_object_encoding<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bytearray) = this.as_bytearray() {
        return Ok((bytearray.object_encoding() as u8).into());
    }

    Ok(Value::Undefined)
}

pub fn set_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut bytearray) = this.as_bytearray_mut() {
        let new_encoding = args.get_u32(activation, 0)?;
        match new_encoding {
            0 => bytearray.set_object_encoding(ObjectEncoding::Amf0),
            3 => bytearray.set_object_encoding(ObjectEncoding::Amf3),
            _ => return Err(make_error_2008(activation, "objectEncoding")),
        }
    }

    Ok(Value::Undefined)
}
