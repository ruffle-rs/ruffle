//! `flash.ui.MouseCursor` builtin

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("The MouseCursor class cannot be constructed.".into())
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "MouseCursor"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init, "<MouseCursor instance initializer>", mc),
        Method::from_builtin(class_init, "<MouseCursor class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const CONSTANTS: &[(&str, &str)] = &[
        ("ARROW", "arrow"),
        ("AUTO", "auto"),
        ("BUTTON", "button"),
        ("HAND", "hand"),
        ("IBEAM", "ibeam"),
    ];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}
