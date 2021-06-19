//! `flash.system.System` class

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.system.System`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("The System class cannot be constructed.".into())
}

/// Implements `flash.system.System`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `System.gc`
pub fn gc<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `System`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.system"), "System"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin_only(instance_init, "<System instance initializer>", mc),
        Method::from_builtin_only(class_init, "<System class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("gc", gc)];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    class
}
