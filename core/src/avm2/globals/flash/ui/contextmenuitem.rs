//! `flash.ui.ContextMenuItem` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        // note: this doesn't propagate arguments to NativeMenuItem correctly
        activation.super_init(this, &[])?;

        // TODO: would be nice to refactor this if we could easily automate default values

        let caption = if let Some(arg) = args.get(0) {
            arg.coerce_to_string(activation)?
        } else {
            // ideally argument validation should not let us call with 0 args
            return Ok(Value::Undefined);
        };
        let separator = if let Some(separator) = args.get(1) {
            separator.coerce_to_boolean()
        } else {
            false
        };
        let enabled = if let Some(enabled) = args.get(2) {
            enabled.coerce_to_boolean()
        } else {
            true
        };
        let visible = if let Some(visible) = args.get(3) {
            visible.coerce_to_boolean()
        } else {
            true
        };
        this.set_property(&Multiname::public("caption"), caption.into(), activation)?;
        this.set_property(
            &Multiname::public("separatorBefore"),
            separator.into(),
            activation,
        )?;
        this.set_property(&Multiname::public("enabled"), enabled.into(), activation)?;
        this.set_property(&Multiname::public("visible"), visible.into(), activation)?;
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

/// Construct `ContextMenuItem`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "ContextMenuItem"),
        Some(QName::new(Namespace::package("flash.display"), "NativeMenuItem").into()),
        Method::from_builtin(instance_init, "<ContextMenuItem instance initializer>", mc),
        Method::from_builtin(class_init, "<ContextMenuItem class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const PUBLIC_INSTANCE_SLOTS: &[(&str, &str, &str)] = &[
        ("caption", "", "String"),
        ("separatorBefore", "", "Boolean"),
        ("visible", "", "Boolean"),
    ];
    write.define_public_slot_instance_traits(PUBLIC_INSTANCE_SLOTS);

    class
}
