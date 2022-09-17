//! `flash.events.IEventDispatcher` builtin

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::{GcCell, MutationContext};

/// Emulates attempts to execute bodiless methods.
pub fn bodiless_method<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err("Cannot execute non-native method without body".into())
}

/// Implements `flash.events.IEventDispatcher`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Construct `IEventDispatcher`'s class.
pub fn create_interface<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "IEventDispatcher"),
        None,
        Method::from_builtin(
            bodiless_method,
            "<IEventDispatcher instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<IEventDispatcher interface initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::INTERFACE);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("addEventListener", bodiless_method),
        ("dispatchEvent", bodiless_method),
        ("hasEventListener", bodiless_method),
        ("removeEventListener", bodiless_method),
        ("willTrigger", bodiless_method),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
