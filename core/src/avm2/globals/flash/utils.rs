//! `flash.utils` namespace

use crate::avm2::object::TObject;
use crate::avm2::QName;
use crate::avm2::{Activation, Error, Object, Value};
use instant::Instant;

pub mod bytearray;
pub mod dictionary;
pub mod proxy;
pub mod timer;

/// `flash.utils.flash_proxy` namespace
pub const NS_FLASH_PROXY: &str = "http://www.adobe.com/2006/actionscript/flash/proxy";

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok((Instant::now()
        .duration_since(activation.context.start_time)
        .as_millis() as u32)
        .into())
}

/// Implements `flash.utils.setInterval`
pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if args.len() < 2 {
        return Err(Error::from("setInterval: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setInterval: not enough arguments")
            .as_object()
            .ok_or("setInterval: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setInterval: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        false,
    )))
}

/// Implements `flash.utils.clearInterval`
pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let id = args
        .get(0)
        .ok_or("clearInterval: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.setTimeout`
pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if args.len() < 2 {
        return Err(Error::from("setTimeout: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setTimeout: not enough arguments")
            .as_object()
            .ok_or("setTimeout: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setTimeout: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        true,
    )))
}

/// Implements `flash.utils.clearTimeout`
pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let id = args
        .get(0)
        .ok_or("clearTimeout: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
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

    Ok(class
        .inner_class_definition()
        .read()
        .name()
        .to_qualified_name(activation.context.gc_context)
        .into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_superclass_name<'gc>(
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
        Ok(super_class
            .inner_class_definition()
            .read()
            .name()
            .to_qualified_name(activation.context.gc_context)
            .into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements native method `flash.utils.getDefinitionByName`
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
    let qname = QName::from_qualified_name(name, activation.context.gc_context);
    appdomain.get_defined_value(activation, qname)
}
