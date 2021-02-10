use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::string::AvmString;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::traits::Trait;
use crate::avm2::method::Method;
use crate::avm2::object::{DomainObject, Object, TObject};
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
    if let Some(mut bytearray) = this.unwrap().as_bytearray_mut(activation.context.gc_context) {
        let byte = args.get(0).cloned().unwrap_or(Value::Undefined).coerce_to_i32(activation)?;
        bytearray.write_byte(byte as u8);
    }
    
    Ok(Value::Undefined)
}

pub fn write_bytes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {    
    if let Some(Value::Object(second_array)) = args.get(0) {
        let combining_storage = second_array.as_bytearray().unwrap().reborrow();
        let combining_bytes = combining_storage.bytes();
        let mut signed_offset = args.get(1).unwrap_or(&Value::Integer(0)).coerce_to_i32(activation).unwrap();
        let mut signed_length = args.get(2).unwrap_or(&Value::Integer(combining_bytes.len()as i32)).coerce_to_i32(activation).unwrap(); 

         // This is so we can convert to an unsigned int without anything crazy happening
        if signed_offset < 0 {signed_offset = 0}
        if signed_length < 0 {signed_length = 0}
        let mut offset = signed_offset as usize;
        let mut length = signed_length as usize;
            
            // The docs say that if the offset or length is out of bounds, Flash will correct the bounds by "clamping them".
            // But in the actual Flash player, it seems to just raise an error. The way Flash raises the errors also seemed very strange to me,
            // so I decided to do what the docs said instead.
        if offset > length {
            offset = length;
        } else if offset > combining_bytes.len(){
            offset = combining_bytes.len();
        }
            
            // Always wrap back to being the size of the buffer were adding
        if length > combining_bytes.len() || length == 0 {
            length = combining_bytes.len();
        }
        if let Some(mut bytearray) = this.unwrap().as_bytearray_mut(activation.context.gc_context) {    
            bytearray.write_bytes(combining_bytes.to_vec());
        }
    }
    
    Ok(Value::Undefined)
}
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this.unwrap().as_bytearray_mut(activation.context.gc_context) {
        let bytes = bytearray.bytes();
        let mut new_string = String::with_capacity(bytes.len());
        for c in bytes.iter(){
            new_string.push(*c as char);
        }
        return Ok(AvmString::new(activation.context.gc_context, new_string).into());
    }
    Ok(Value::Undefined)
}

pub fn position<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut bytearray) = this.unwrap().as_bytearray_mut(activation.context.gc_context) {
        return Ok(Value::Unsigned(bytearray.position() as u32));
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

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeByte"),
        Method::from_builtin(write_byte),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "writeBytes"),
        Method::from_builtin(write_bytes),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "toString"),
        Method::from_builtin(to_string),
    ));
    
    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::as3_namespace(), "position"),
        Method::from_builtin(position),
    ));

    class
}
