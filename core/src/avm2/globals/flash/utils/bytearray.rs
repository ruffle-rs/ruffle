use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::bytearray::{ByteArrayStorage, CompressionAlgorithm, Endian, ObjectEncoding};
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{bytearray_allocator, ArrayObject, ByteArrayObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::character::Character;
use crate::string::AvmString;
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
use flash_lso::amf0::read::AMF0Decoder;
use flash_lso::amf3::read::AMF3Decoder;
use flash_lso::types::Value as AmfValue;
use gc_arena::{GcCell, MutationContext};

pub fn deserialize_value<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: &AmfValue,
) -> Result<Value<'gc>, Error> {
    Ok(match value {
        AmfValue::Undefined => Value::Undefined,
        AmfValue::Null => Value::Null,
        AmfValue::Bool(b) => Value::Bool(*b),
        AmfValue::Integer(i) => Value::Integer(*i),
        AmfValue::Number(n) => Value::Number(*n),
        AmfValue::String(s) => Value::String(AvmString::new_utf8(activation.context.gc_context, s)),
        AmfValue::ByteArray(bytes) => {
            let storage = ByteArrayStorage::from_vec(bytes.clone());
            let bytearray = ByteArrayObject::from_storage(activation, storage)?;
            bytearray.into()
        }
        AmfValue::StrictArray(values) => {
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value(activation, value)?));
            }
            let storage = ArrayStorage::from_storage(arr);
            let array = ArrayObject::from_storage(activation, storage)?;
            array.into()
        }
        AmfValue::ECMAArray(values, elements, _) => {
            // First lets create an array out of `values` (dense portion), then we add the elements onto it.
            let mut arr: Vec<Option<Value<'gc>>> = Vec::with_capacity(values.len());
            for value in values {
                arr.push(Some(deserialize_value(activation, value)?));
            }
            let storage = ArrayStorage::from_storage(arr);
            let mut array = ArrayObject::from_storage(activation, storage)?;
            // Now lets add each element as a property
            for element in elements {
                array.set_property(
                    &QName::new(
                        Namespace::public(),
                        AvmString::new_utf8(activation.context.gc_context, element.name()),
                    )
                    .into(),
                    deserialize_value(activation, element.value())?,
                    activation,
                )?;
            }
            array.into()
        }
        AmfValue::Object(properties, _class_definition) => {
            let obj_class = activation.avm2().classes().object;
            let mut obj = obj_class.construct(activation, &[])?;
            for property in properties {
                obj.set_property(
                    &QName::new(
                        Namespace::public(),
                        AvmString::new_utf8(activation.context.gc_context, property.name()),
                    )
                    .into(),
                    deserialize_value(activation, property.value())?,
                    activation,
                )?;
            }
            obj.into()
            // TODO: Handle class_defintion
        }
        // TODO: Dictionary, Vector, XML, Date, etc...
        _ => Value::Undefined,
    })
}

/// Implements `flash.utils.ByteArray`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        let class_object = this
            .instance_of()
            .ok_or("Attempted to construct ByteArray on a bare object")?;
        if let Some((movie, id)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class_object)
        {
            if let Some(lib) = activation.context.library.library_for_movie(movie) {
                if let Some(Character::BinaryData(binary_data)) = lib.character_by_id(id) {
                    let mut byte_array = this
                        .as_bytearray_mut(activation.context.gc_context)
                        .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
                    byte_array.clear();
                    byte_array.write_bytes(binary_data.as_ref())?;
                    byte_array.set_position(0);
                }
            }
        }
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
    if let Some(this) = this {
        let bytearray = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let offset = args
            .get(1)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;
        let length = args
            .get(2)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;
        if !Object::ptr_eq(this, bytearray) {
            // The ByteArray we are reading from is different than the ByteArray we are writing to,
            // so we are allowed to borrow both at the same time without worrying about a panic

            let ba_read = bytearray
                .as_bytearray()
                .ok_or("ArgumentError: Parameter must be a bytearray")?;
            let to_write = ba_read.read_at(
                // If length is 0, lets read the remaining bytes of ByteArray from the supplied offset
                if length != 0 {
                    length
                } else {
                    ba_read.len().saturating_sub(offset)
                },
                offset,
            )?;

            if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
                bytearray.write_bytes(to_write)?;
            }
        } else if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            // The ByteArray we are reading from is the same as the ByteArray we are writing to,
            // so we only need to borrow once, and we can use `write_bytes_within` to write bytes from our own ByteArray
            let amnt = if length != 0 {
                length
            } else {
                bytearray.len().saturating_sub(offset)
            };
            bytearray.write_bytes_within(offset, amnt)?;
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
        let bytearray = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let offset = args
            .get(1)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;
        let length = args
            .get(2)
            .unwrap_or(&Value::Unsigned(0))
            .coerce_to_u32(activation)? as usize;

        if !Object::ptr_eq(this, bytearray) {
            if let Some(bytearray_read) = this.as_bytearray() {
                let to_write = bytearray_read.read_bytes(
                    // If length is 0, lets read the remaining bytes of ByteArray
                    if length != 0 {
                        length
                    } else {
                        bytearray_read.bytes_available()
                    },
                )?;

                let mut ba_write = bytearray
                    .as_bytearray_mut(activation.context.gc_context)
                    .ok_or("ArgumentError: Parameter must be a bytearray")?;

                ba_write.write_at(to_write, offset)?;
            }
        } else if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let amnt = if length != 0 {
                length
            } else {
                bytearray.bytes_available()
            };
            let pos = bytearray.position();
            bytearray.write_at_within(pos, amnt, offset)?;
        }
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
                // NOTE: there is a bug on old Flash Player (e.g. v11.3); if the string to
                // write ends with an unpaired high surrogate, the routine bails out and nothing
                // is written.
                // The bug is fixed on newer FP versions (e.g. v32), but the fix isn't SWF-version-gated.
                bytearray.write_utf(&utf_string.to_utf8_lossy())?;
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
            return Ok(
                AvmString::new_utf8_bytes(activation.context.gc_context, bytearray.read_utf()?).into(),
            );
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
            return Ok(AvmString::new_utf8_bytes(activation.context.gc_context, bytearray.bytes()).into());
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
            let endian = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_string(activation)?;
            if &endian == b"bigEndian" {
                bytearray.set_endian(Endian::Big);
            } else if &endian == b"littleEndian" {
                bytearray.set_endian(Endian::Little);
            } else {
                return Err("Parameter type must be one of the accepted values.".into());
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
            return Ok(AvmString::new_utf8(
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
            let encoder =
                Encoding::for_label(charset_label.to_utf8_lossy().as_bytes()).unwrap_or(UTF_8);
            let utf8 = string.to_utf8_lossy();
            let (encoded_bytes, _, _) = encoder.encode(&utf8);
            bytearray.write_bytes(&encoded_bytes)?;
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
            let encoder =
                Encoding::for_label(charset_label.to_utf8_lossy().as_bytes()).unwrap_or(UTF_8);
            let (decoded_str, _, _) = encoder.decode(bytes);
            return Ok(AvmString::new_utf8(activation.context.gc_context, decoded_str).into());
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
            bytearray.write_bytes(string.to_utf8_lossy().as_bytes())?;
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
            let buffer = bytearray.compress(algorithm.parse()?)?;
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
            let buffer = bytearray.decompress(algorithm.parse()?)?;
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

pub fn read_object<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            let bytes = bytearray.read_at(bytearray.bytes_available(), bytearray.position())?;
            let (bytes_left, value) = match bytearray.object_encoding() {
                ObjectEncoding::Amf0 => {
                    let mut decoder = AMF0Decoder::default();
                    let (extra, amf) = decoder
                        .parse_single_element(bytes)
                        .map_err(|_| "Error: Invalid object")?;
                    (extra.len(), deserialize_value(activation, &amf)?)
                }
                ObjectEncoding::Amf3 => {
                    let mut decoder = AMF3Decoder::default();
                    let (extra, amf) = decoder
                        .parse_single_element(bytes)
                        .map_err(|_| "Error: Invalid object")?;
                    (extra.len(), deserialize_value(activation, &amf)?)
                }
            };

            bytearray.set_position(bytearray.len() - bytes_left);
            return Ok(value);
        }
    }

    Ok(Value::Undefined)
}

pub fn object_encoding<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(bytearray) = this.as_bytearray() {
            return Ok((bytearray.object_encoding() as u8).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn set_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut bytearray) = this.as_bytearray_mut(activation.context.gc_context) {
            let new_encoding = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)?;
            match new_encoding {
                0 => bytearray.set_object_encoding(ObjectEncoding::Amf0),
                3 => bytearray.set_object_encoding(ObjectEncoding::Amf3),
                _ => return Err("Parameter type must be one of the accepted values.".into()),
            }
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
    write.set_instance_allocator(bytearray_allocator);

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
        ("readObject", read_object),
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
        (
            "objectEncoding",
            Some(object_encoding),
            Some(set_object_encoding),
        ),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    // TODO: This property should have a setter
    const CONSTANTS: &[(&str, u32)] = &[("defaultObjectEncoding", 3)];

    write.define_public_constant_uint_class_traits(CONSTANTS);

    class
}
