//! `flash.accessibility.AccessibilityProperties` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `flash.accessibility.AccessibilityProperties`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(
            Namespace::package("flash.accessibility"),
            "AccessibilityProperties",
        ),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(
            instance_init,
            "<AccessibilityProperties instance initializer>",
            mc,
        ),
        Method::from_builtin(
            class_init,
            "<AccessibilityProperties class initializer>",
            mc,
        ),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_SLOTS: &[(&str, &str, &str)] = &[
        ("description", "", "String"),
        ("forceSimple", "", "Boolean"),
        ("name", "", "String"),
        ("noAutoLabeling", "", "Boolean"),
        ("shortcut", "", "String"),
        ("silent", "", "Boolean"),
    ];

    write.define_public_slot_instance_traits(PUBLIC_INSTANCE_SLOTS);

    class
}
