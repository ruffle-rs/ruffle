//! `flash.display.SWFVersion` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.SWFVersion`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.SWFVersion`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `SWFVersion`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "SWFVersion"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<SWFVersion instance initializer>", mc),
        Method::from_builtin(class_init, "<SWFVersion class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, u32)] = &[
        ("FLASH1", 1),
        ("FLASH2", 2),
        ("FLASH3", 3),
        ("FLASH4", 4),
        ("FLASH5", 5),
        ("FLASH6", 6),
        ("FLASH7", 7),
        ("FLASH8", 8),
        ("FLASH9", 9),
        ("FLASH10", 10),
        ("FLASH11", 11),
        ("FLASH12", 12),
    ];
    write.define_public_constant_uint_class_traits(CONSTANTS);

    class
}
