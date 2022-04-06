//! `flash.net.URLLoaderDataFormat` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.net.URLLoaderDataFormat`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.net.URLLoaderDataFormat`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `URLLoaderDataFormat`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.net"), "URLLoaderDataFormat"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(
            instance_init,
            "<URLLoaderDataFormat instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<URLLoaderDataFormat class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const CONSTANTS: &[(&str, &str)] = &[
        ("BINARY", "binary"),
        ("TEXT", "text"),
        ("VARIABLES", "variables"),
    ];

    write.define_public_constant_string_class_traits(CONSTANTS);
    class
}
