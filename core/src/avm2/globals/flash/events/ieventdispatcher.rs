//! `flash.events.IEventDispatcher` builtin

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Emulates attempts to execute bodiless methods.
pub fn bodiless_method<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Cannot execute non-native method without body".into())
}

/// Implements `flash.events.IEventDispatcher`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `IEventDispatcher`'s class.
pub fn create_interface<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "IEventDispatcher"),
        None,
        Method::from_builtin(bodiless_method),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::Interface.into());
    write.define_instance_trait(Trait::from_method(
        QName::dynamic_name("addEventListener"),
        Method::from_builtin(bodiless_method),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::dynamic_name("dispatchEvent"),
        Method::from_builtin(bodiless_method),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::dynamic_name("hasEventListener"),
        Method::from_builtin(bodiless_method),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::dynamic_name("removeEventListener"),
        Method::from_builtin(bodiless_method),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::dynamic_name("willTrigger"),
        Method::from_builtin(bodiless_method),
    ));

    class
}
