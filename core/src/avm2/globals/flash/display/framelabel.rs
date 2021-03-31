//! `flash.display.FrameLabel` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.FrameLabel`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let name = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(activation)?;
    let frame = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_i32(activation)?;

    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "name"),
            name.into(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "frame"),
            frame.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.FrameLabel`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `FrameLabel.name`.
pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "name"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.frame`.
pub fn frame<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "frame"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Construct `FrameLabel`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "FrameLabel"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "name"),
        Method::from_builtin(name),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "frame"),
        Method::from_builtin(frame),
    ));

    class
}
