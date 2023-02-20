//! `Namespace` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::object::{namespace_allocator, Object};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use crate::avm2_stub_constructor;
use gc_arena::GcCell;

/// Implements `Namespace`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_constructor!(activation.context.stub_tracker, "Namespace");
    Err("Namespace constructor is a stub.".into())
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_constructor!(activation.context.stub_tracker, "Namespace");
    Err("Namespace constructor is a stub.".into())
}

/// Implements `Namespace`'s native instance initializer.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, args)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Namespace`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Construct `Namespace`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace, "Namespace"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
        Method::from_builtin(instance_init, "<Namespace instance initializer>", mc),
        Method::from_builtin(class_init, "<Namespace class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(namespace_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Namespace native instance initializer>",
        mc,
    ));
    write.set_call_handler(Method::from_builtin(
        class_call,
        "<Namespace call handler>",
        mc,
    ));

    class
}
