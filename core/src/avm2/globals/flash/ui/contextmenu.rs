//! `flash.ui.ContextMenu` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
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
        log::warn!("flash.ui.ContextMenu is a stub");
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

fn hide_built_in_items<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // TODO: replace this by a proper implementation.
    log::warn!("flash.ui.ContextMenu is a stub");
    activation
        .context
        .stage
        .set_show_menu(&mut activation.context, false);

    Ok(Value::Undefined)
}

fn is_supported<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // TODO: return true when replaced by proper implementation
    Ok(false.into())
}

/// Construct `ContextMenu`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "ContextMenu"),
        Some(QName::new(Namespace::package("flash.display"), "NativeMenu").into()),
        Method::from_builtin(instance_init, "<ContextMenu instance initializer>", mc),
        Method::from_builtin(class_init, "<ContextMenu class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const PUBLIC_CLASS_PROPERTIES: &[(&str, Option<NativeMethodImpl>, Option<NativeMethodImpl>)] =
        &[("isSupported", Some(is_supported), None)];
    write.define_public_builtin_class_properties(mc, PUBLIC_CLASS_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("hideBuiltInItems", hide_built_in_items)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
