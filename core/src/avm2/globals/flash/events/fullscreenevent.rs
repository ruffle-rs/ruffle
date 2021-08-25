use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.FullScreenEvent`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // Event uses the first three parameters
    }
    Ok(Value::Undefined)
}

/// Implements `flash.events.FullScreenEvent`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn fullscreen<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("FullScreenEvent.fullscreen - not implemented");

    Ok(Value::Undefined)
}

pub fn interactive<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("FullScreenEvent.interactive - not implemented");

    Ok(Value::Undefined)
}

/// Construct `FullScreenEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "FullScreenEvent"),
        Some(QName::new(Namespace::package("flash.events"), "ActivityEvent").into()),
        Method::from_builtin(instance_init, "<FullScreenEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<FullScreenEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("fullScreen", Some(fullscreen), None),
        ("interactive", Some(interactive), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[
        ("FULL_SCREEN", "fullScreen"),
        (
            "FULL_SCREEN_INTERACTIVE_ACCEPTED",
            "fullScreenInteractiveAccepted",
        ),
    ];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}
