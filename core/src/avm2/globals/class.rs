//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ClassObject, Object, ScriptObject, TObject};
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
    Ok(Value::Undefined)
}

/// Construct `Class` and `Class.prototype`, respectively.
///
/// This function additionally returns the class initializer method for `Class`,
/// which must be called before user code runs.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    globals: Object<'gc>,
    superclass: Object<'gc>,
    super_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Result<(Object<'gc>, Object<'gc>, Object<'gc>), Error> {
    let class_class = Class::new(
        QName::new(Namespace::public(), "Class"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(
            instance_init,
            "<Class instance initializer>",
            activation.context.gc_context,
        ),
        Method::from_builtin(
            class_init,
            "<Class class initializer>",
            activation.context.gc_context,
        ),
        activation.context.gc_context,
    );

    let scope = Scope::push_scope(globals.get_scope(), globals, activation.context.gc_context);
    let proto = ScriptObject::object(activation.context.gc_context, super_proto);

    let (class_object, cinit) = ClassObject::from_builtin_class(
        activation.context.gc_context,
        Some(superclass),
        class_class,
        Some(scope),
        proto,
        fn_proto,
    )?;

    Ok((class_object, proto, cinit))
}
