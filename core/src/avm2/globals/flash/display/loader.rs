//! `flash.display.Loader` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::value::Value;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{Error, Object};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.Loader`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.display.Loader`'s instance constructor.
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

// TODO: this is entirely stubbed, just to get addEventListener to not crash
fn content_loader_info<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Loader::contentLoaderInfo is a stub");
    Ok(this.unwrap().into())
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Loader"),
        Some(
            QName::new(
                Namespace::package("flash.display"),
                "DisplayObjectContainer",
            )
            .into(),
        ),
        Method::from_builtin(instance_init, "<Loader instance initializer>", mc),
        Method::from_builtin(class_init, "<Loader class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("contentLoaderInfo", Some(content_loader_info), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
