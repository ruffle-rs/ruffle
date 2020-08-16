//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;

/// Implements `Class`'s instance initializer.
///
/// Notably, you cannot construct new classes this way, so this returns an
/// error.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Classes cannot be constructed.".into())
}

/// Implement's `Class`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Classes cannot be constructed.".into())
}

/// Construct `Class` and `Class.prototype`, respectively.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    super_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> (Object<'gc>, Object<'gc>) {
    let class_class = Class::new(
        QName::new(Namespace::public_namespace(), "Class"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        activation.context.gc_context,
    );

    let globals = activation.avm2().globals();
    let scope = Scope::push_scope(globals.get_scope(), globals, activation.context.gc_context);
    let proto = ScriptObject::prototype(
        activation.context.gc_context,
        super_proto,
        class_class,
        Some(scope),
    );

    let constr = FunctionObject::from_builtin_constr(
        activation.context.gc_context,
        instance_init,
        proto,
        fn_proto,
    )
    .unwrap();

    (constr, proto)
}
