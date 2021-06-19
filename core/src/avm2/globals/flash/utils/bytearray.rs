use crate::avm2::activation::Activation;
use crate::avm2::bytearray::{CompressionAlgorithm, Endian};
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{bytearray_deriver, Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
use gc_arena::{GcCell, MutationContext};
use std::str::FromStr;

/// Implements `flash.utils.ByteArray`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.utils.ByteArray`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Writes a single byte to the bytearray
pub fn write_byte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let byte = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;
            bytearray.write_bytes(&[byte as u8])?;
        }
    }

    Ok(Value::Undefined)
}

/// Writes multiple bytes to the bytearray from another bytearray
pub fn write_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(Value::Object(second_array)) = args.get(0) {
        let combining_bytes = match second_array.as_bytearray() {
            Some(b) => b.bytes().clone(),
            None => return Err("ArgumentError: Parameter must be a bytearray".into()),
        };

        let offset = args
            .get(1)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;
        let length = args
            .get(2)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;

        // In the docs it says "If offset or length is out of range, they are clamped to the beginning and end of the bytes array."
        // However, in the actual flash player, it seems to just raise an error.
        if offset + length > combining_bytes.len() {
            return Err("EOFError: Reached EOF".into());
        }
        if let Some(this) = this {
            if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
                bytearray.write_bytes(if length != 0 {
                    &combining_bytes[offset..length + offset]
                } else {
                    &combining_bytes[offset..]
                })?;
            }
        }
    }

    Ok(Value::Undefined)
}

// Reads the bytes from the current bytearray into another bytearray
pub fn read_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let current_bytes = this
            .as_bytearray_mut(activation.context.gc_context)
            .unwrap()
            .bytes()
            .clone();
        let position = this
            .as_bytearray_mut(activation.context.gc_context)
            .unwrap()
            .position();
        let mut merging_offset = 0;
        if let Some(Value::Object(second_array)) = args.get(0) {
            let offset = args
                .get(1)
                .unwrap_or(&Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;
            let length = args
                .get(2)
                .unwrap_or(&Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;

            if position + length > current_bytes.len() {
                return Err("EOFError: Reached EOF".into());
            }
            if let Some(mut merging_storage) =
                second_array.as_bytearray_mut(activation.context.gc_context)
            {
                let to_write = if length != 0 {
                    &current_bytes[position..length + position]
                } else {
                    &current_bytes[position..]
                };
                merging_offset = to_write.len();
                merging_storage.write_at(to_write, offset)?;
            } else {
                return Err("ArgumentError: Parameter must be a bytearray".into());
            }
        }
        this.as_bytearray_mut(activation.context.gc_context)
            .unwrap()
            .add_position(merging_offset);
    }

    Ok(Value::Undefined)
}
pub fn write_utf<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            if let Some(utf_string) = args.get(0) {
                let utf_string = utf_string.coerce_to_string(activation)?;
                bytearray.write_utf(utf_string.as_str())?;
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn read_utf<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(AvmString::new(activation.context.gc_context, bytearray.read_utf()?).into());
        }
    }

    Ok(Value::Undefined)
}
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            let (new_string, _, _) = UTF_8.decode(bytearray.bytes());
            return Ok(AvmString::new(activation.context.gc_context, new_string).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            bytearray.clear();
            bytearray.shrink_to_fit();
        }
    }

    Ok(Value::Undefined)
}

pub fn position<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.position() as u32));
        }
    }

    Ok(Value::Undefined)
}

pub fn set_position<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            let num = args
                .get(0)
                .unwrap_or(&Value::Integer(0))
                .coerce_to_u32(activation)?;
            bytearray.set_position(num as usize);
        }
    }

    Ok(Value::Undefined)
}

pub fn bytes_available<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.bytes_available() as u32));
        }
    }

    Ok(Value::Undefined)
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.len() as u32));
        }
    }

    Ok(Value::Undefined)
}

pub fn set_length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let len = args
                .get(0)
                .unwrap_or(&Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;
            bytearray.set_length(len);
        }
    }

    Ok(Value::Undefined)
}

pub fn endian<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(match bytearray.endian() {
                Endian::Big => "bigEndian".into(),
                Endian::Little => "littleEndian".into(),
            });
        }
    }

    Ok(Value::Undefined)
}

pub fn set_endian<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            match args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?
                .as_str()
            {
                "bigEndian" => bytearray.set_endian(Endian::Big),
                "littleEndian" => bytearray.set_endian(Endian::Little),
                _ => return Err("Parameter type must be one of the accepted values.".into()),
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn read_short<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Integer(bytearray.read_short()? as i32));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_short<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.read_unsigned_short()? as u32));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_double<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Number(bytearray.read_double()?));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_float<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Number(bytearray.read_float()? as f64));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_int<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Integer(bytearray.read_int()?));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_int<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.read_unsigned_int()?));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_boolean<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Bool(bytearray.read_boolean()?));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_byte<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Integer(bytearray.read_byte()? as i32));
        }
    }

    Ok(Value::Undefined)
}

pub fn read_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            let len = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)?;
            return Ok(AvmString::new(
                activation.context.gc_context,
                String::from_utf8_lossy(bytearray.read_bytes(len as usize)?),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

pub fn read_unsigned_byte<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok(Value::Unsigned(bytearray.read_unsigned_byte()? as u32));
        }
    }

    Ok(Value::Undefined)
}

pub fn write_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_number(activation)?;
            bytearray.write_float(num as f32)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_double<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_number(activation)?;
            bytearray.write_double(num)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_boolean<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args.get(0).unwrap_or(&Value::Undefined).coerce_to_boolean();
            bytearray.write_boolean(num)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;
            bytearray.write_int(num)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)?;
            bytearray.write_unsigned_int(num)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_short<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let num = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;
            bytearray.write_short(num as i16)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn write_multibyte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let string = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?;
            let charset_label = args
                .get(1)
                .unwrap_or(&"UTF-8".into())
                .coerce_to_string(activation)?;
            let encoder = Encoding::for_label(charset_label.as_bytes()).unwrap_or(UTF_8);
            let (encoded_bytes, _, _) = encoder.encode(string.as_str());
            bytearray.write_bytes(&encoded_bytes.into_owned())?;
        }
    }

    Ok(Value::Undefined)
}

pub fn read_multibyte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            let len = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)?;
            let charset_label = args
                .get(1)
                .unwrap_or(&"UTF-8".into())
                .coerce_to_string(activation)?;
            let bytes = bytearray.read_bytes(len as usize)?;
            let encoder = Encoding::for_label(charset_label.as_bytes()).unwrap_or(UTF_8);
            let (decoded_str, _, _) = encoder.decode(bytes);
            return Ok(AvmString::new(activation.context.gc_context, decoded_str).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn write_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let string = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?;
            bytearray.write_bytes(string.as_bytes())?;
        }
    }

    Ok(Value::Undefined)
}

pub fn compress<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let algorithm = args
                .get(0)
                .unwrap_or(&"zlib".into())
                .coerce_to_string(activation)?;
            let buffer = bytearray.compress(CompressionAlgorithm::from_str(algorithm.as_str())?)?;
            bytearray.clear();
            bytearray.write_bytes(&buffer)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn uncompress<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let algorithm = args
                .get(0)
                .unwrap_or(&"zlib".into())
                .coerce_to_string(activation)?;
            let buffer =
                bytearray.decompress(CompressionAlgorithm::from_str(algorithm.as_str())?)?;
            bytearray.clear();
            bytearray.write_bytes(&buffer)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn deflate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let buffer = bytearray.compress(CompressionAlgorithm::Deflate)?;
            bytearray.clear();
            bytearray.write_bytes(&buffer)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn inflate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let buffer = bytearray.decompress(CompressionAlgorithm::Deflate)?;
            bytearray.clear();
            bytearray.write_bytes(&buffer)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.utils"), "ByteArray"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<ByteArray instance initializer>", mc),
        Method::from_builtin(class_init, "<ByteArray class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_deriver(bytearray_deriver);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("writeByte", write_byte),
        ("writeBytes", write_bytes),
        ("readBytes", read_bytes),
        ("toString", to_string),
        ("readShort", read_short),
        ("writeShort", write_short),
        ("readUnsignedShort", read_unsigned_short),
        ("readDouble", read_double),
        ("writeDouble", write_double),
        ("readFloat", read_float),
        ("writeFloat", write_float),
        ("readInt", read_int),
        ("writeInt", write_int),
        ("readUnsignedInt", read_unsigned_int),
        ("writeUnsignedInt", write_unsigned_int),
        ("readBoolean", read_boolean),
        ("writeBoolean", write_boolean),
        ("readByte", read_byte),
        ("readUnsignedByte", read_unsigned_byte),
        ("writeUTF", write_utf),
        ("readUTF", read_utf),
        ("clear", clear),
        ("compress", compress),
        ("uncompress", uncompress),
        ("inflate", inflate),
        ("deflate", deflate),
        ("writeMultiByte", write_multibyte),
        ("readMultiByte", read_multibyte),
        ("writeUTFBytes", write_utf_bytes),
        ("readUTFBytes", read_utf_bytes),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bytesAvailable", Some(bytes_available), None),
        ("length", Some(length), Some(set_length)),
        ("position", Some(position), Some(set_position)),
        ("endian", Some(endian), Some(set_endian)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
