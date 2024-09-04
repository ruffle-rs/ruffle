//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::eval_error;
use crate::avm2::globals::array::resolve_array_hole;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{function_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

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
            None,
            None,
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
            None,
            None,
        )
        .into(),
        activation,
    )?;
    function_proto.set_string_property_local(
        "toString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", activation.context.gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    function_proto.set_string_property_local(
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", activation.context.gc_context),
            scope,
            None,
            None,
            None,
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
    function_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "toString".into(),
        false,
    );
    function_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "toLocaleString".into(),
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

/// Implements `Function.prototype.toString` and `Function.prototype.toLocaleString`
fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok("function Function() {}".into())
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
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object_i_class: Class<'gc>,
    class_i_class: Class<'gc>,
) -> Class<'gc> {
    let gc_context = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let function_i_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Function"),
        Some(object_i_class),
        Method::from_builtin(instance_init, "<Function instance initializer>", gc_context),
        gc_context,
    );

    let function_c_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Function$"),
        Some(class_i_class),
        Method::from_builtin(class_init, "<Function class initializer>", gc_context),
        gc_context,
    );
    function_c_class.set_attributes(gc_context, ClassAttributes::FINAL);

    function_i_class.set_c_class(gc_context, function_c_class);
    function_c_class.set_i_class(gc_context, function_i_class);

    // Fixed traits (in AS3 namespace)
    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("call", call), ("apply", apply)];
    function_i_class.define_builtin_instance_methods(
        gc_context,
        namespaces.as3,
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
    function_i_class.define_builtin_instance_properties(
        gc_context,
        namespaces.public_all(),
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    function_c_class.define_constant_int_instance_traits(
        namespaces.public_all(),
        CONSTANTS_INT,
        activation,
    );

    function_i_class.set_instance_allocator(gc_context, function_allocator);
    function_i_class.set_call_handler(
        gc_context,
        Method::from_builtin(class_call, "<Function call handler>", gc_context),
    );

    function_i_class.mark_traits_loaded(activation.context.gc_context);
    function_i_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    function_c_class.mark_traits_loaded(activation.context.gc_context);
    function_c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    function_i_class
}
