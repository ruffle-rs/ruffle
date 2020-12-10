//! `flash.display.SimpleButton` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.SimpleButton`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let up_state = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?
        .as_display_object()
        .unwrap();
    let over_state = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?
        .as_display_object()
        .unwrap();
    let down_state = args
        .get(2)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?
        .as_display_object()
        .unwrap();
    let hit_test_state = args
        .get(3)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?
        .as_display_object()
        .unwrap();

    if let Some(mut this) = this {
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "upState"),
            up_state.object2(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "overState"),
            over_state.object2(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "downState"),
            down_state.object2(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "hitTestState"),
            hit_test_state.object2(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.SimpleButton`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `SimpleButton.downState`.
pub fn down_state<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "downState"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.enabled`.
pub fn enabled<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "enabled"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.hitTestState`.
pub fn hit_test_state<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "hitTestState"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.overState`.
pub fn over_state<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "overState"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.soundTransform`.
pub fn sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "soundTransform"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.trackAsMenu`.
pub fn track_as_menu<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "trackAsMenu"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.upState`.
pub fn up_state<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "upState"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.useHandCursor`.
pub fn use_hand_cursor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        return this.get_property(
            this,
            &QName::new(Namespace::Private("ruffle".into()), "useHandCursor"),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Construct `SimpleButton`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "SimpleButton"),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "downState"),
        Method::from_builtin(down_state),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "enabled"),
        Method::from_builtin(enabled),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "hitTestState"),
        Method::from_builtin(hit_test_state),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "overState"),
        Method::from_builtin(over_state),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "soundTransform"),
        Method::from_builtin(sound_transform),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "trackAsMenu"),
        Method::from_builtin(track_as_menu),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "upState"),
        Method::from_builtin(up_state),
    ));

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "useHandCursor"),
        Method::from_builtin(use_hand_cursor),
    ));

    class
}
