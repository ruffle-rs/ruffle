//! `flash.display.StageDisplayState` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.StageDisplayState`'s instance constructor.
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

/// Implements `flash.display.StageDisplayState`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `StageDisplayState`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "StageDisplayState"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FULL_SCREEN"),
        QName::new(Namespace::public(), "String").into(),
        Some("fullScreen".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "FULL_SCREEN_INTERACTIVE"),
        QName::new(Namespace::public(), "String").into(),
        Some("fullScreenInteractive".into()),
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "NORMAL"),
        QName::new(Namespace::public(), "String").into(),
        Some("normal".into()),
    ));

    class
}
