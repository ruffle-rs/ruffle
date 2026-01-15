//! `Boolean` class impl

use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "valueOf" => method(value_of; DONT_ENUM | DONT_DELETE);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(constructor, Some(function), super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

/// `Boolean` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args
        .get(0)
        .is_some_and(|value| value.as_bool(activation.swf_version()));
    // Called from a constructor, populate `this`.
    this.set_native(activation.gc(), NativeObject::Bool(value));

    Ok(this.into())
}

/// `Boolean` function
fn function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // If called as a function, return the value.
    // Boolean() with no argument returns undefined.
    Ok(args
        .get(0)
        .map(|value| value.as_bool(activation.swf_version()))
        .map_or(Value::Undefined, Value::Bool))
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Must be a bool.
    // Boolean.prototype.toString.call(x) returns undefined for non-bools.
    if let NativeObject::Bool(value) = this.native() {
        return Ok(Value::from(match value {
            true => istr!("true"),
            false => istr!("false"),
        }));
    }

    Ok(Value::Undefined)
}

pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Must be a bool.
    // Boolean.prototype.valueOf.call(x) returns undefined for non-bools.
    if let NativeObject::Bool(value) = this.native() {
        return Ok(value.into());
    }

    Ok(Value::Undefined)
}
