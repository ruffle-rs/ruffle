//! `Boolean` class impl

use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::FunctionObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{NativeObject, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
    "valueOf" => method(value_of; DONT_ENUM | DONT_DELETE);
};

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
pub fn boolean_function<'gc>(
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

pub fn create_boolean_object<'gc>(
    context: &mut StringContext<'gc>,
    boolean_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        constructor,
        Some(boolean_function),
        fn_proto,
        boolean_proto,
    )
}

/// Creates `Boolean.prototype`.
pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let boolean_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, boolean_proto, fn_proto);
    boolean_proto.into()
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
