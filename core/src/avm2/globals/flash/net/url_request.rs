//! `flash.net.URLRequest` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.net.URLRequest`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.net.URLRequest`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(url) = args.get(0) {
            this.set_property(
                &QName::new(Namespace::public(), "url").into(),
                *url,
                activation,
            )?;
        }
    }
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.net"), "URLRequest"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<URLRequest instance initializer>", mc),
        Method::from_builtin(class_init, "<URLRequest class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    // NOTE - when implementing properties (e.g. `contentType`, `data`, etc.)
    // be sure to also check for them in `UrlLoader`
    const PUBLIC_INSTANCE_SLOTS: &[(&str, &str, &str)] = &[("url", "", "String")];
    write.define_public_slot_instance_traits(PUBLIC_INSTANCE_SLOTS);

    class
}
