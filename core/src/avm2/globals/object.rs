//! Object builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::error;
use crate::avm2::object::{Object, ScriptObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use crate::string::AvmString;

/// Implements `Object`'s custom constructor, called when ActionScript code runs
/// `new Object(...)` directly.
pub fn object_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(arg) = args.get_optional(0) {
        if !matches!(arg, Value::Undefined | Value::Null) {
            return Ok(arg);
        }
    }

    let constructed_object = ScriptObject::new_object(activation);
    Ok(constructed_object.into())
}

/// Implements `Object.prototype.toString`
pub fn _to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = args.get_value(0);

    if let Some(this) = this.as_object() {
        Ok(this.to_string(activation.gc()).into())
    } else {
        let class_name = this.instance_class(activation).name().local_name();

        Ok(AvmString::new_utf8(activation.gc(), format!("[object {class_name}]")).into())
    }
}

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_value(0).coerce_to_string(activation)?;

    if let Some(this) = this.as_object() {
        Ok(this.has_own_property_string(name, activation)?.into())
    } else {
        let name = Multiname::new(activation.avm2().find_public_namespace(), name);

        Ok(this.has_trait(activation, &name).into())
    }
}

/// `Object.prototype.isPrototypeOf`
pub fn is_prototype_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_object() {
        let mut target_proto = args.get_value(0);

        while let Value::Object(proto) = target_proto {
            if Object::ptr_eq(this, proto) {
                return Ok(true.into());
            }

            target_proto = proto.proto().map(|o| o.into()).unwrap_or(Value::Undefined);
        }
    }

    Ok(false.into())
}

/// `Object.prototype.propertyIsEnumerable`
pub fn property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_object() {
        let name = args.get_value(0).coerce_to_string(activation)?;

        Ok(this.property_is_enumerable(name).into())
    } else {
        Ok(false.into())
    }
}

/// `Object.prototype.setPropertyIsEnumerable`
pub fn _set_property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = args.get_value(0);

    let name = args.get_string(activation, 1);

    if let Some(this) = this.as_object() {
        let is_enum = args.get_bool(2);
        this.set_local_property_is_enumerable(activation.gc(), name, is_enum);
    } else {
        let instance_class = this.instance_class(activation);
        let multiname = Multiname::new(activation.avm2().find_public_namespace(), name);

        return Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::InvalidWrite,
            &multiname,
            instance_class,
        ));
    }

    Ok(Value::Undefined)
}
