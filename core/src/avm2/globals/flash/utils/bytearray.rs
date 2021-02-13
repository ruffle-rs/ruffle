use crate::avm2::activation::Activation;
use crate::avm2::bytearray::Endian;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
use gc_arena::{GcCell, MutationContext};

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
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let byte = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        bytearray.write_byte(byte as u8);
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
        let combining_storage = second_array.as_bytearray().unwrap().reborrow();
        let combining_bytes = combining_storage.bytes();
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
            log::error!("ByteArray: Reached EOF");
            return Ok(Value::Undefined);
        }

        if let Some(mut bytearray) = this
            .unwrap()
            .as_bytearray_mut(activation.context.gc_context)
        {
            bytearray.write_bytes(if length != 0 {
                &combining_bytes[offset..length + offset]
            } else {
                &combining_bytes[offset..]
            });
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
    let mut merging_buffer: Vec<u8> = Vec::new();
    let mut offset = 0;
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let combining_bytes = bytearray.bytes();
        offset = args
            .get(1)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;
        let length = args
            .get(2)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;

        if bytearray.position() + length > combining_bytes.len() {
            log::error!("ByteArray: Reached EOF");
            return Ok(Value::Undefined);
        }

        merging_buffer = if length != 0 {
            combining_bytes[bytearray.position()..length + bytearray.position()].to_vec()
        } else {
            combining_bytes[bytearray.position()..].to_vec()
        };
        {
            bytearray.add_position(merging_buffer.len());
        }
    }
    // We borrow the 2 bytearrays seperately in case they are trying to add 2 of the same bytearrays together (would panic otherwise)
    if let Some(Value::Object(second_array)) = args.get(0) {
        let mut merging_storage = second_array
            .as_bytearray_mut(activation.context.gc_context)
            .unwrap();
        // Offset should not be greater then the buffer
        if merging_storage.bytes().len() < offset {
            return Ok(Value::Undefined);
        }
        merging_storage.write_bytes_at(&merging_buffer, offset);
    }
    Ok(Value::Undefined)
}

pub fn write_utf<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Some(utf_string) = args.get(0) {
            let utf_string = utf_string.coerce_to_string(activation)?;
            bytearray.write_utf(&utf_string.as_str());
        }
    }
    Ok(Value::Undefined)
}

pub fn read_utf<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(utf_string) = bytearray.read_utf() {
            return Ok(AvmString::new(activation.context.gc_context, utf_string).into());
        }
    }
    Ok(Value::Undefined)
}
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bytearray) = this.unwrap().as_bytearray() {
        let bytes = bytearray.bytes();
        let (new_string, _, _) = UTF_8.decode(bytes);
        return Ok(AvmString::new(activation.context.gc_context, new_string).into());
    }
    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        bytearray.clear();
    }
    Ok(Value::Undefined)
}

pub fn position<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bytearray) = this.unwrap().as_bytearray() {
        return Ok(Value::Unsigned(bytearray.position() as u32));
    }
    Ok(Value::Undefined)
}

pub fn set_position<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Integer(0))
            .coerce_to_u32(activation)?;
        bytearray.set_position(num as usize);
    }
    Ok(Value::Undefined)
}

pub fn bytes_available<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bytearray) = this.unwrap().as_bytearray() {
        return Ok(Value::Unsigned(
            if bytearray.position() > bytearray.bytes().len() {
                0
            } else {
                (bytearray.bytes().len() - bytearray.position()) as u32
            },
        ));
    }
    Ok(Value::Undefined)
}

pub fn bytes_length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bytearray) = this.unwrap().as_bytearray() {
        return Ok(Value::Unsigned(bytearray.bytes().len() as u32));
    }
    Ok(Value::Undefined)
}

pub fn set_bytes_length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let len = args
            .get(0)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)
            .unwrap() as usize;
        bytearray.resize(len);
    }
    Ok(Value::Undefined)
}

pub fn endian<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bytearray) = this.unwrap().as_bytearray() {
        return Ok(match bytearray.endian() {
            Endian::Big => AvmString::new(activation.context.gc_context, "bigEndian").into(),
            Endian::Little => AvmString::new(activation.context.gc_context, "littleEndian").into(),
        });
    }
    Ok(Value::Undefined)
}

pub fn set_endian<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        match args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?
            .as_str()
        {
            "bigEndian" => bytearray.set_endian(Endian::Big),
            "littleEndian" => bytearray.set_endian(Endian::Little),
            _ => log::error!("Parameter type must be one of the accepted values."),
        }
    }
    Ok(Value::Undefined)
}

pub fn read_short<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_short() {
            return Ok(Value::Integer(num as i32));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_unsigned_short<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_unsigned_short() {
            return Ok(Value::Unsigned(num as u32));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_double<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_double() {
            return Ok(Value::Number(num));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_float() {
            return Ok(Value::Number(num as f64));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_int() {
            return Ok(Value::Integer(num));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_unsigned_int() {
            return Ok(Value::Unsigned(num));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_boolean<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_boolean() {
            return Ok(Value::Bool(num));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_byte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_byte() {
            return Ok(Value::Integer(num as i32));
        }
    }
    Ok(Value::Undefined)
}

pub fn read_utf_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Some(Value::Integer(len)) = args.get(0) {
            if *len < 0 {
                log::error!("ByteArray: Did not get proper length");
                return Ok(Value::Undefined);
            }
            if let Ok(bytes) = bytearray.read_exactly(*len as usize) {
                return Ok(AvmString::new(
                    activation.context.gc_context,
                    String::from_utf8_lossy(&bytes),
                )
                .into());
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn read_unsigned_byte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(num) = bytearray.read_unsigned_byte() {
            return Ok(Value::Unsigned(num as u32));
        }
    }
    Ok(Value::Undefined)
}

pub fn write_float<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_number(activation)?;
        bytearray.write_float(num as f32);
    }
    Ok(Value::Undefined)
}

pub fn write_double<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_number(activation)?;
        bytearray.write_double(num);
    }
    Ok(Value::Undefined)
}

pub fn write_boolean<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args.get(0).unwrap_or(&Value::Undefined).coerce_to_boolean();
        bytearray.write_boolean(num);
    }
    Ok(Value::Undefined)
}

pub fn write_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        bytearray.write_int(num);
    }
    Ok(Value::Undefined)
}

pub fn write_unsigned_int<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        bytearray.write_unsigned_int(num);
    }
    Ok(Value::Undefined)
}

pub fn write_short<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let num = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        bytearray.write_short(num as i16);
    }
    Ok(Value::Undefined)
}

pub fn write_multibyte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let string = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;
        let charset_label = args
            .get(1)
            .unwrap_or(&AvmString::new(activation.context.gc_context, "UTF-8").into())
            .coerce_to_string(activation)?;
        let encoder = Encoding::for_label(charset_label.as_bytes()).unwrap_or(UTF_8);
        let (encoded_bytes, _, _) = encoder.encode(string.as_str());
        bytearray.write_bytes(&encoded_bytes.into_owned());
    }
    Ok(Value::Undefined)
}

pub fn read_multibyte<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let len = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let charset_label = args
            .get(1)
            .unwrap_or(&AvmString::new(activation.context.gc_context, "UTF-8").into())
            .coerce_to_string(activation)?;
        if let Ok(bytes) = bytearray.read_exactly(len as usize) {
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
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        let string = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;
        bytearray.write_bytes(string.as_bytes());
    }
    Ok(Value::Undefined)
}

pub fn compress<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Value::String(string) = args.get(0).unwrap_or(&Value::Undefined) {
            let compressed = match string.as_str() {
                "zlib" => bytearray.zlib_compress(),
                "deflate" => bytearray.deflate_compress(),
                &_ => return Ok(Value::Undefined),
            };
            if let Ok(buffer) = compressed {
                bytearray.clear();
                bytearray.write_bytes(&buffer);
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn uncompress<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Value::String(string) = args.get(0).unwrap_or(&Value::Undefined) {
            let compressed = match string.as_str() {
                "zlib" => bytearray.zlib_decompress(),
                "deflate" => bytearray.deflate_decompress(),
                &_ => return Ok(Value::Undefined),
            };
            if let Ok(buffer) = compressed {
                bytearray.clear();
                bytearray.write_bytes(&buffer);
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn deflate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(buffer) = bytearray.deflate_compress() {
            bytearray.clear();
            bytearray.write_bytes(&buffer);
        }
    }
    Ok(Value::Undefined)
}

pub fn inflate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this
        .unwrap()
        .as_bytearray_mut(activation.context.gc_context)
    {
        if let Ok(buffer) = bytearray.deflate_decompress() {
            bytearray.clear();
            bytearray.write_bytes(&buffer);
        }
    }
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.utils"), "ByteArray"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    class.write(mc).set_attributes(ClassAttributes::SEALED);

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeByte"),
        Method::from_builtin(write_byte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeBytes"),
        Method::from_builtin(write_bytes),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readBytes"),
        Method::from_builtin(read_bytes),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "toString"),
        Method::from_builtin(to_string),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readShort"),
        Method::from_builtin(read_short),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeShort"),
        Method::from_builtin(write_short),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readUnsignedShort"),
        Method::from_builtin(read_unsigned_short),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readDouble"),
        Method::from_builtin(read_double),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeDouble"),
        Method::from_builtin(write_double),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readFloat"),
        Method::from_builtin(read_float),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeFloat"),
        Method::from_builtin(write_float),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readInt"),
        Method::from_builtin(read_int),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeInt"),
        Method::from_builtin(write_int),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readUnsignedInt"),
        Method::from_builtin(read_unsigned_int),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeUnsignedInt"),
        Method::from_builtin(write_unsigned_int),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readBoolean"),
        Method::from_builtin(read_boolean),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeBoolean"),
        Method::from_builtin(write_boolean),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readByte"),
        Method::from_builtin(read_byte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readUnsignedByte"),
        Method::from_builtin(read_unsigned_byte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeUTF"),
        Method::from_builtin(write_utf),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readUTF"),
        Method::from_builtin(read_utf),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "clear"),
        Method::from_builtin(clear),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "compress"),
        Method::from_builtin(compress),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "uncompress"),
        Method::from_builtin(uncompress),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "inflate"),
        Method::from_builtin(inflate),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "deflate"),
        Method::from_builtin(deflate),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeMultiByte"),
        Method::from_builtin(write_multibyte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readMultiByte"),
        Method::from_builtin(read_multibyte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeUTFBytes"),
        Method::from_builtin(write_utf_bytes),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "readUTFBytes"),
        Method::from_builtin(read_utf_bytes),
    ));

    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::as3_namespace(), "bytesAvailable"),
        Method::from_builtin(bytes_available),
    ));

    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::as3_namespace(), "length"),
        Method::from_builtin(bytes_length),
    ));

    class.write(mc).define_instance_trait(Trait::from_setter(
        QName::new(Namespace::as3_namespace(), "length"),
        Method::from_builtin(set_bytes_length),
    ));

    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::as3_namespace(), "position"),
        Method::from_builtin(position),
    ));

    class.write(mc).define_instance_trait(Trait::from_setter(
        QName::new(Namespace::as3_namespace(), "position"),
        Method::from_builtin(set_position),
    ));

    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::as3_namespace(), "endian"),
        Method::from_builtin(endian),
    ));
    class.write(mc).define_instance_trait(Trait::from_setter(
        QName::new(Namespace::as3_namespace(), "endian"),
        Method::from_builtin(set_endian),
    ));

    class
}
