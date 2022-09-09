//! `flash.system.Capabilities` class

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.system.Capabilities`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("The Capabilities class cannot be constructed.".into())
}

/// Implements `flash.system.Capabilities`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// `os` static property.
pub fn os<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::String("Linux 5.10.49".into())) // Temporary
}

/// `playerType` static property.
pub fn player_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::String("StandAlone".into())) // Temporary
}

/// `version` static property.
pub fn version<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::String("LNX 32,0,0,465".into())) // Temporary
}

/// Construct `Capabilities`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.system"), "Capabilities"),
        Some(Multiname::public("Object")),
        Method::from_builtin(instance_init, "<Capabilities instance initializer>", mc),
        Method::from_builtin(class_init, "<Capabilities class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_CLASS_TRAITS: &[(&str, Option<NativeMethodImpl>, Option<NativeMethodImpl>)] = &[
        ("os", Some(os), None),
        ("playerType", Some(player_type), None),
        ("version", Some(version), None),
    ];

    write.define_public_builtin_class_properties(mc, PUBLIC_CLASS_TRAITS);

    class
}
