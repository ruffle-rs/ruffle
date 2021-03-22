//! `flash.display.SWFVersion` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
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
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH1"),
        QName::new(Namespace::public(), "uint").into(),
        Some(1.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH2"),
        QName::new(Namespace::public(), "uint").into(),
        Some(2.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH3"),
        QName::new(Namespace::public(), "uint").into(),
        Some(3.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH4"),
        QName::new(Namespace::public(), "uint").into(),
        Some(4.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH5"),
        QName::new(Namespace::public(), "uint").into(),
        Some(5.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH6"),
        QName::new(Namespace::public(), "uint").into(),
        Some(6.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH7"),
        QName::new(Namespace::public(), "uint").into(),
        Some(7.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH8"),
        QName::new(Namespace::public(), "uint").into(),
        Some(8.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH9"),
        QName::new(Namespace::public(), "uint").into(),
        Some(9.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH10"),
        QName::new(Namespace::public(), "uint").into(),
        Some(10.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH11"),
        QName::new(Namespace::public(), "uint").into(),
        Some(11.into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FLASH12"),
        QName::new(Namespace::public(), "uint").into(),
        Some(12.into()),
    ));

    class
}
