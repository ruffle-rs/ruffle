//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::{eval_error, type_error};
use crate::avm2::globals::array::resolve_array_hole;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::object::{FunctionObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;

use gc_arena::{Gc, GcCell};

/// Implements `Function`'s custom constructor.
/// This is used when ActionScript manually calls 'new Function()',
/// which produces a dummy function that just returns `Value::Undefined`
/// when called.
pub fn function_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !args.is_empty() {
        return Err(Error::AvmError(eval_error(
            activation,
            "Error #1066: The form function('function body') is not supported.",
            1066,
        )?));
    }

    let mc = activation.gc();

    let dummy = Gc::new(
        mc,
        NativeMethod {
            method: |_, _, _| Ok(Value::Undefined),
            name: "<Empty Function>",
            signature: vec![],
            resolved_signature: GcCell::new(mc, None),
            return_type: None,
            is_variadic: true,
        },
    );

    let function_object = FunctionObject::from_method(
        activation,
        Method::Native(dummy),
        activation.create_scopechain(),
        None,
        None,
        None,
    );

    Ok(function_object.into())
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation
        .avm2()
        .classes()
        .function
        .construct(activation, args)
}

pub fn _init_function_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Set Function's prototype, register it in SystemClasses, and initialize
    // Object's prototype. This method is called from AS during builtins
    // initialization.
    let function_class_object = this.as_object().unwrap().as_class_object().unwrap();

    activation.avm2().system_classes.as_mut().unwrap().function = function_class_object;

    let function_proto = function_class_object
        .construct(activation, &[])?
        .as_object()
        .unwrap();
    function_class_object.link_prototype(activation, function_proto);

    crate::avm2::globals::object::init_object_prototype(activation)?;

    Ok(Value::Undefined)
}

/// Implements `Function.prototype.call`
pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let func = func.as_object().unwrap().as_function_object().unwrap();

    let this = args.get_value(0);

    if args.len() > 1 {
        Ok(func.call(activation, this, &args[1..])?)
    } else {
        Ok(func.call(activation, this, &[])?)
    }
}

/// Implements `Function.prototype.apply`
pub fn apply<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let func = func.as_object().unwrap().as_function_object().unwrap();

    let this = args.get_value(0);

    let arg_array = args.get_value(1);
    let resolved_args = if !matches!(arg_array, Value::Undefined | Value::Null) {
        if let Some(array_object) = arg_array.as_object().and_then(|o| o.as_array_object()) {
            let arg_storage = array_object
                .as_array_storage()
                .unwrap()
                .iter()
                .collect::<Vec<_>>();

            let mut resolved_args = Vec::with_capacity(arg_storage.len());
            for (i, v) in arg_storage.iter().enumerate() {
                resolved_args.push(resolve_array_hole(activation, array_object.into(), i, *v)?);
            }

            resolved_args
        } else {
            return Err(Error::AvmError(type_error(
                activation,
                "Error #1116: second argument to Function.prototype.apply must be an array.",
                1116,
            )?));
        }
    } else {
        // Passing null or undefined results in the function being called with no arguments passed
        Vec::new()
    };

    func.call(activation, this, &resolved_args)
}

pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_function_object() {
        return Ok(this.executable().num_parameters().into());
    }

    Ok(Value::Undefined)
}

pub fn get_prototype<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(function) = this.as_function_object() {
        if let Some(proto) = function.prototype() {
            return Ok(proto.into());
        } else {
            return Ok(Value::Undefined);
        }
    }

    Ok(Value::Undefined)
}

pub fn set_prototype<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(function) = this.as_function_object() {
        let new_proto = args.get_value(0).as_object();
        function.set_prototype(new_proto, activation.gc());
    }

    Ok(Value::Undefined)
}
