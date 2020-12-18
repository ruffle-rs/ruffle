//! `flash.display.Scene` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.Scene`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let labels = args.get(1).cloned().unwrap_or(Value::Undefined);
        let num_frames = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;

        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "name"),
            name.into(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "labels"),
            labels,
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "numFrames"),
            num_frames.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Scene`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Scene.labels`.
pub fn labels<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "labels"),
            activation,
        )
    } else {
        Ok(Value::Undefined)
    }
}

/// Implements `Scene.name`.
pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "name"),
            activation,
        )
    } else {
        Ok(Value::Undefined)
    }
}

/// Implements `Scene.numFrames`.
pub fn num_frames<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "numFrames"),
            activation,
        )
    } else {
        Ok(Value::Undefined)
    }
}

/// Construct `Scene`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Scene"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "labels"),
        Method::from_builtin(labels),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "name"),
        Method::from_builtin(name),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "numFrames"),
        Method::from_builtin(num_frames),
    ));

    class
}
