//! `flash.utils` namespace

use crate::avm2::object::TObject;
use crate::avm2::string::AvmString;
use crate::avm2::QName;
use crate::avm2::{Activation, Error, Object, Value};

pub mod bytearray;
pub mod compression_algorithm;
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

    Ok(AvmString::new(
        activation.context.gc_context,
        obj.as_class()
            .ok_or("This object does not have a class")?
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

    if let Some(super_class) = obj.superclass_object() {
        Ok(AvmString::new(
            activation.context.gc_context,
            super_class
                .as_class()
                .ok_or("This object does not have a class")?
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
    let globals = activation.scope().map(|s| s.read().globals());
    if let Some(appdomain) = globals.and_then(|g| g.as_application_domain()) {
        let name = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;
        let qname = QName::from_qualified_name(&name, activation.context.gc_context);
        return appdomain.get_defined_value(activation, qname);
    }
    Ok(Value::Undefined)
}
