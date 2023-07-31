//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::error::eval_error;
use crate::avm2::globals::array::resolve_array_hole;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{function_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use gc_arena::GcCell;

/// Implements `Function`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !args.is_empty() {
        return Err(Error::AvmError(eval_error(
            activation,
            "Error #1066: The form function('function body') is not supported.",
            1066,
        )?));
    }

    activation.super_init(this, &[])?;

    Ok(Value::Undefined)
}

pub fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation
        .avm2()
        .classes()
        .function
        .construct(activation, args)?
        .into())
}

/// Implements `Function`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let this_class = this.as_class_object().unwrap();
    let function_proto = this_class.prototype();

    function_proto.set_string_property_local(
        "call",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(call, "call", activation.context.gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    function_proto.set_string_property_local(
        "apply",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(apply, "apply", activation.context.gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    function_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "call".into(),
        false,
    );
    function_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "apply".into(),
        false,
    );

    Ok(Value::Undefined)
}

/// Implements `Function.prototype.call`
fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = args.get(0).copied().unwrap_or(Value::Null);

    if args.len() > 1 {
        Ok(func.call(this, &args[1..], activation)?)
    } else {
        Ok(func.call(this, &[], activation)?)
    }
}

/// Implements `Function.prototype.apply`
fn apply<'gc>(
    activation: &mut Activation<'_, 'gc>,
    func: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = args.get(0).copied().unwrap_or(Value::Null);

    let arg_array = args.get(1).cloned().unwrap_or(Value::Undefined).as_object();
    let resolved_args = if let Some(arg_array) = arg_array {
        let arg_storage: Vec<Option<Value<'gc>>> = arg_array
            .as_array_storage()
            .map(|a| a.iter().collect())
            .ok_or_else(|| {
                Error::from("Second parameter of apply must be an array or undefined")
            })?;

        let mut resolved_args = Vec::with_capacity(arg_storage.len());
        for (i, v) in arg_storage.iter().enumerate() {
            resolved_args.push(resolve_array_hole(activation, arg_array, i, *v)?);
        }

        resolved_args
    } else {
        Vec::new()
    };

    func.call(this, &resolved_args, activation)
}

fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_function_object() {
        return Ok(this.num_parameters().into());
    }

    Ok(Value::Undefined)
}

fn prototype<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(function) = this.as_function_object() {
        if let Some(proto) = function.prototype() {
            return Ok(proto.into());
        } else {
            return Ok(Value::Undefined);
        }
    }

    Ok(Value::Undefined)
}

fn set_prototype<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(function) = this.as_function_object() {
        let new_proto = args.get(0).unwrap_or(&Value::Undefined).as_object();
        function.set_prototype(new_proto, activation.context.gc_context);
    }

    Ok(Value::Undefined)
}

/// Construct `Function`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let gc_context = activation.context.gc_context;
    let function_class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Function"),
        Some(Multiname::new(
            activation.avm2().public_namespace_base_version,
            "Object",
        )),
        Method::from_builtin(instance_init, "<Function instance initializer>", gc_context),
        Method::from_builtin(class_init, "<Function class initializer>", gc_context),
        gc_context,
    );

    let mut write = function_class.write(gc_context);

    // Fixed traits (in AS3 namespace)
    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("call", call), ("apply", apply)];
    write.define_builtin_instance_methods(
        gc_context,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("prototype", Some(prototype), Some(set_prototype)),
        ("length", Some(length), None),
    ];
    write.define_builtin_instance_properties(
        gc_context,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    write.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CONSTANTS_INT,
        activation,
    );

    write.set_instance_allocator(function_allocator);
    write.set_call_handler(Method::from_builtin(
        class_call,
        "<Function call handler>",
        gc_context,
    ));

    function_class
}
