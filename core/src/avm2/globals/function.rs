//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::array::resolve_array_hole;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut function_proto = this
            .get_property(this, &QName::dynamic_name("prototype").into(), activation)?
            .coerce_to_object(activation)?;

        function_proto.install_dynamic_property(
            activation.context.gc_context,
            QName::new(Namespace::as3_namespace(), "call"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(call, "call", activation.context.gc_context),
                None,
                None,
            )
            .into(),
        )?;
        function_proto.install_dynamic_property(
            activation.context.gc_context,
            QName::new(Namespace::as3_namespace(), "apply"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(apply, "apply", activation.context.gc_context),
                None,
                None,
            )
            .into(),
        )?;
    }
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
    let base_proto = this.and_then(|that| that.instance_of());

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
    let base_proto = this.and_then(|that| that.instance_of());

    if let Some(func) = func {
        let arg_array = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation);
        let resolved_args = if let Ok(arg_array) = arg_array {
            let arg_storage: Vec<Option<Value<'gc>>> = arg_array
                .as_array_storage()
                .map(|a| a.iter().collect())
                .ok_or_else(|| {
                    Error::from("Second parameter of apply must be an array or undefined")
                })?;

            let mut resolved_args = Vec::with_capacity(arg_storage.len());
            for (i, v) in arg_storage.iter().enumerate() {
                resolved_args.push(resolve_array_hole(activation, arg_array, i, v.clone())?);
            }

            resolved_args
        } else {
            Vec::new()
        };

        Ok(func.call(this, &resolved_args, activation, base_proto)?)
    } else {
        Err("Not a callable function".into())
    }
}

/// Construct `Function`'s class.
pub fn create_class<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let function_class = Class::new(
        QName::new(Namespace::public(), "Function"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Function instance initializer>", gc_context),
        Method::from_builtin(class_init, "<Function class initializer>", gc_context),
        gc_context,
    );

    function_class
}
