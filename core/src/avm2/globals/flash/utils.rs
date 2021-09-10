//! `flash.utils` namespace

use crate::avm2::object::TObject;
use crate::avm2::QName;
use crate::avm2::{Activation, Error, Object, Value};
use crate::string::AvmString;

pub mod bytearray;
pub mod compression_algorithm;
pub mod dictionary;
pub mod endian;

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok((activation.context.navigator.time_since_launch().as_millis() as u32).into())
}

/// Implements `flash.utils.getQualifiedClassName`
pub fn get_qualified_class_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    Ok(AvmString::new(
        activation.context.gc_context,
        class
            .inner_class_definition()
            .read()
            .name()
            .to_qualified_name(),
    )
    .into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_super_class_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    if let Some(super_class) = class.superclass_object() {
        Ok(AvmString::new(
            activation.context.gc_context,
            super_class
                .inner_class_definition()
                .read()
                .name()
                .to_qualified_name(),
        )
        .into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements `flash.utils.getDefinitionByName`
pub fn get_definition_by_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let appdomain = activation.caller_domain();
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let qname = QName::from_qualified_name(&name, activation.context.gc_context);
    appdomain.get_defined_value(activation, qname)
}
