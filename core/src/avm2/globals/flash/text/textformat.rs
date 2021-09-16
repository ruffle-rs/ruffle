//! `flash.text.TextFormat` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.text.TextFormat`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        this.set_property(
            this,
            &QName::new(Namespace::public(), "font").into(),
            args.get(0).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "size").into(),
            args.get(1).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "color").into(),
            args.get(2).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "bold").into(),
            args.get(3).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "italic").into(),
            args.get(4).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "underline").into(),
            args.get(5).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "url").into(),
            args.get(6).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "target").into(),
            args.get(7).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "align").into(),
            args.get(8).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "leftMargin").into(),
            args.get(9).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "rightMargin").into(),
            args.get(10).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "indent").into(),
            args.get(11).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "leading").into(),
            args.get(12).cloned().unwrap_or(Value::Null),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.text.TextFormat`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `TextFormat`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.text"), "TextFormat"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<TextFormat instance initializer>", mc),
        Method::from_builtin(class_init, "<TextFormat class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const ITEMS: &[(&str, &str)] = &[
        ("align", "String"),
        ("blockIndent", "Object"),
        ("bold", "Object"),
        ("bullet", "Object"),
        ("color", "Object"),
        ("font", "String"),
        ("indent", "Object"),
        ("italic", "Object"),
        ("kerning", "Object"),
        ("leading", "Object"),
        ("leftMargin", "Object"),
        ("letterSpacing", "Object"),
        ("rightMargin", "Object"),
        ("size", "Object"),
        ("tabStops", "Array"),
        ("target", "String"),
        ("underline", "Object"),
        ("url", "String"),
    ];
    for &(name, type_name) in ITEMS {
        write.define_instance_trait(Trait::from_slot(
            QName::new(Namespace::public(), name),
            QName::new(Namespace::public(), type_name).into(),
            None,
        ));
    }

    class
}
