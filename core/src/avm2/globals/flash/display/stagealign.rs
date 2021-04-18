//! `flash.display.StageAlign` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.StageAlign`'s instance constructor.
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

/// Implements `flash.display.StageAlign`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `StageAlign`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "StageAlign"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "BOTTOM"),
        QName::new(Namespace::public(), "String").into(),
        Some("B".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "BOTTOM_LEFT"),
        QName::new(Namespace::public(), "String").into(),
        Some("BL".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "BOTTOM_RIGHT"),
        QName::new(Namespace::public(), "String").into(),
        Some("BR".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "LEFT"),
        QName::new(Namespace::public(), "String").into(),
        Some("L".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "RIGHT"),
        QName::new(Namespace::public(), "String").into(),
        Some("R".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "TOP"),
        QName::new(Namespace::public(), "String").into(),
        Some("T".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "TOP_LEFT"),
        QName::new(Namespace::public(), "String").into(),
        Some("TL".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "TOP_RIGHT"),
        QName::new(Namespace::public(), "String").into(),
        Some("TR".into()),
    ));

    class
}
