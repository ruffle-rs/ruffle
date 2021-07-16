//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::array::resolve_array_hole;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::GcCell;

/// Implements `Function`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `Function`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Function.prototype.call`
fn call<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    func: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = args
        .get(0)
        .and_then(|v| v.coerce_to_object(activation).ok());
    let base_proto = this.and_then(|that| that.proto());

    if let Some(func) = func {
        if args.len() > 1 {
            Ok(func.call(this, &args[1..], activation, base_proto)?)
        } else {
            Ok(func.call(this, &[], activation, base_proto)?)
        }
    } else {
        Err("Not a callable function".into())
    }
}

/// Implements `Function.prototype.apply`
fn apply<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    func: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = args
        .get(0)
        .and_then(|v| v.coerce_to_object(activation).ok());
    let base_proto = this.and_then(|that| that.proto());

    if let Some(func) = func {
        let arg_array = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let arg_storage: Vec<Option<Value<'gc>>> = arg_array
            .as_array_storage()
            .map(|a| a.iter().collect())
            .ok_or_else(|| Error::from("Second parameter of apply must be an array"))?;

        let mut resolved_args = Vec::with_capacity(arg_storage.len());
        for (i, v) in arg_storage.iter().enumerate() {
            resolved_args.push(resolve_array_hole(activation, arg_array, i, v.clone())?);
        }

        Ok(func.call(this, &resolved_args, activation, base_proto)?)
    } else {
        Err("Not a callable function".into())
    }
}

/// Construct `Function` and `Function.prototype`, respectively.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    globals: Object<'gc>,
    proto: Object<'gc>,
) -> (Object<'gc>, Object<'gc>, GcCell<'gc, Class<'gc>>) {
    let function_class = Class::new(
        QName::new(Namespace::public(), "Function"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        activation.context.gc_context,
    );

    let scope = Scope::push_scope(globals.get_scope(), globals, activation.context.gc_context);
    let mut function_proto = ScriptObject::prototype(
        activation.context.gc_context,
        proto,
        function_class,
        Some(scope),
    );

    function_proto.install_method(
        activation.context.gc_context,
        QName::new(Namespace::as3_namespace(), "call"),
        0,
        FunctionObject::from_builtin(activation.context.gc_context, call, function_proto),
    );
    function_proto.install_method(
        activation.context.gc_context,
        QName::new(Namespace::as3_namespace(), "apply"),
        0,
        FunctionObject::from_builtin(activation.context.gc_context, apply, function_proto),
    );

    let constr = FunctionObject::from_builtin_constr(
        activation.context.gc_context,
        instance_init,
        proto,
        function_proto,
    )
    .unwrap();

    (constr, function_proto, function_class)
}
