//! Function prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{ExecutionName, ExecutionReason};
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Object, Value};
use crate::string::AvmString;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "call" => method(call; DONT_ENUM | DONT_DELETE);
    "apply" => method(apply; DONT_ENUM | DONT_DELETE);
};

/// Constructs the `Function` class.
///
/// Since Object and Function are so heavily intertwined, this function does
/// not allocate an object to store either proto. Instead, they must be provided
/// through the `DeclContext`.
pub fn create_class<'gc>(context: &mut DeclContext<'_, 'gc>) -> SystemClass<'gc> {
    let class = context.native_class_with_proto(constructor, Some(function), context.fn_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    class
}

/// Implements `new Function()`
fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args.get(0).copied().unwrap_or_else(|| this.into()))
}

/// Implements `Function()`
fn function<'gc>(
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
        Value::Undefined | Value::Null => activation.global_object(),
        this_val => this_val.coerce_to_object(activation),
    };
    let empty = [];
    let args = match myargs.len() {
        0 => &empty,
        1 => &empty,
        _ => &myargs[1..],
    };

    // NOTE: does not use `Object::call`, as `super()` only works with direct calls.
    match func.as_function() {
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
        Value::Undefined | Value::Null => activation.global_object(),
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
    match func.as_function() {
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
