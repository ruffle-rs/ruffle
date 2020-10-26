//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::GcCell;

/// Implements `Function`'s instance initializer.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

/// Construct `Function` and `Function.prototype`, respectively.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    globals: Object<'gc>,
    proto: Object<'gc>,
) -> (Object<'gc>, Object<'gc>, GcCell<'gc, Class<'gc>>) {
    let function_class = Class::new(
        QName::new(Namespace::public_namespace(), "Function"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
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

    let constr = FunctionObject::from_builtin_constr(
        activation.context.gc_context,
        instance_init,
        proto,
        function_proto,
    )
    .unwrap();

    (constr, function_proto, function_class)
}
