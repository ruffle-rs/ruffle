//! Function prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{ExecutionName, ExecutionReason};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, Value};
use crate::string::{AvmString, StringContext};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "call" => method(call; DONT_ENUM | DONT_DELETE);
    "apply" => method(apply; DONT_ENUM | DONT_DELETE);
};

/// Implements `new Function()`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args.get(0).copied().unwrap_or_else(|| this.into()))
}

/// Implements `Function()`
pub fn function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args.get(0).copied().unwrap_or_else(|| {
        // Calling `Function()` seems to give a prototypeless bare object.
        Object::new(&activation.context.strings, None).into()
    }))
}

/// Implements `Function.prototype.call`
pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Object<'gc>,
    myargs: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match myargs.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => activation.context.avm1.global_object(),
        this_val => this_val.coerce_to_object(activation),
    };
    let empty = [];
    let args = match myargs.len() {
        0 => &empty,
        1 => &empty,
        _ => &myargs[1..],
    };

    // NOTE: does not use `Object::call`, as `super()` only works with direct calls.
    match func.as_executable() {
        Some(exec) => exec.exec(
            ExecutionName::Static("[Anonymous]"),
            activation,
            this.into(),
            1,
            args,
            ExecutionReason::FunctionCall,
            func,
        ),
        _ => Ok(Value::Undefined),
    }
}

/// Implements `Function.prototype.apply`
pub fn apply<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Object<'gc>,
    myargs: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match myargs.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => activation.context.avm1.global_object(),
        this_val => this_val.coerce_to_object(activation),
    };
    let args_object = myargs.get(1).cloned().unwrap_or(Value::Undefined);
    let length = match args_object {
        Value::Object(a) => a.length(activation)? as usize,
        _ => 0,
    };

    let mut child_args = Vec::with_capacity(length);
    while child_args.len() < length {
        let args = args_object.coerce_to_object(activation);
        // TODO: why don't this use args_object.array_element?
        let next_arg = format!("{}", child_args.len());
        let next_arg = args.get(AvmString::new_utf8(activation.gc(), next_arg), activation)?;

        child_args.push(next_arg);
    }

    // NOTE: does not use `Object::call`, as `super()` only works with direct calls.
    match func.as_executable() {
        Some(exec) => exec.exec(
            ExecutionName::Static("[Anonymous]"),
            activation,
            this.into(),
            1,
            &child_args,
            ExecutionReason::FunctionCall,
            func,
        ),
        _ => Ok(Value::Undefined),
    }
}

/// Partially construct `Function.prototype`.
///
/// `__proto__` and other cross-linked properties of this object will *not*
/// be defined here. The caller of this function is responsible for linking
/// them in order to obtain a valid ECMAScript `Function` prototype. The
/// returned object is also a bare object, which will need to be linked into
/// the prototype of `Object`.
pub fn create_proto<'gc>(context: &mut StringContext<'gc>, proto: Object<'gc>) -> Object<'gc> {
    let function_proto = Object::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, function_proto, function_proto);
    function_proto
}
