//! `String` impl

use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::{activation::Activation, traits::Trait};
use crate::string_utils;
use gc_arena::{GcCell, MutationContext};

/// Implements `String`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if let Some(mut value) = this.as_primitive_mut(activation.context.gc_context) {
            *value = args
                .get(0)
                .unwrap_or(&Value::String("".into()))
                .coerce_to_string(activation)?
                .into();
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `length` property's getter
fn length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            return Ok(s.encode_utf16().count().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `String.charAt`
fn char_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Value::String(s) = this.value_of(activation.context.gc_context)? {
            // This function takes Number, so if we use coerce_to_i32 instead of coerce_to_number, the value may overflow.
            let n = args
                .get(0)
                .unwrap_or(&Value::Number(0.0))
                .coerce_to_number(activation)?;
            if n < 0.0 {
                return Ok("".into());
            }

            let index = if !n.is_nan() { n as usize } else { 0 };
            let ret = s
                .encode_utf16()
                .nth(index)
                .map(|c| string_utils::utf16_code_unit_to_char(c).to_string())
                .unwrap_or_default();
            return Ok(AvmString::new(activation.context.gc_context, ret).into());
        }
    }

    Ok(Value::Undefined)
}

/// Construct `String`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "String"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "length"),
        Method::from_builtin(length),
    ));

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "charAt"),
        Method::from_builtin(char_at),
    ));

    class
}
