//! `flash.text.TextFormat` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
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
            &QName::new(Namespace::public(), "font"),
            args.get(0).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "size"),
            args.get(1).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "color"),
            args.get(2).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "bold"),
            args.get(3).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "italic"),
            args.get(4).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "underline"),
            args.get(5).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "url"),
            args.get(6).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "target"),
            args.get(7).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "align"),
            args.get(8).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "leftMargin"),
            args.get(9).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "rightMargin"),
            args.get(10).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "indent"),
            args.get(11).cloned().unwrap_or(Value::Null),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "leading"),
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
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "align"),
        QName::new(Namespace::public(), "String").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "blockIndent"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "bold"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "bullet"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "color"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "font"),
        QName::new(Namespace::public(), "String").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "indent"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "italic"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "kerning"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "leading"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "leftMargin"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "letterSpacing"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "rightMargin"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "size"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "tabStops"),
        QName::new(Namespace::public(), "Array").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "target"),
        QName::new(Namespace::public(), "String").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "underline"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "url"),
        QName::new(Namespace::public(), "String").into(),
        None,
    ));

    class
}
