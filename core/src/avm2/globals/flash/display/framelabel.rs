//! `flash.display.FrameLabel` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{define_indirect_properties, Class};
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::Method;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.FrameLabel`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let name = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(activation)?;
    let frame = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_i32(activation)?;

    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        this.set_property(
            &Multiname::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "name"),
            name.into(),
            activation,
        )?;
        this.set_property(
            &Multiname::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "frame"),
            frame.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.FrameLabel`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}
/// Construct `FrameLabel`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "FrameLabel"),
        Some(Multiname::new(
            Namespace::package("flash.events"),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<FrameLabel instance initializer>", mc),
        Method::from_builtin(class_init, "<FrameLabel class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    define_indirect_properties!(write, mc, [("name", "", "String"), ("frame", "", "int")]);
    class
}
