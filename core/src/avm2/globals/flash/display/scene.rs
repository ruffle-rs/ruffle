//! `flash.display.Scene` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
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
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "name").into(),
            name.into(),
            activation,
        )?;
        this.set_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "labels").into(),
            labels,
            activation,
        )?;
        this.set_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "numFrames").into(),
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
    if let Some(this) = this {
        this.get_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "labels").into(),
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
    if let Some(this) = this {
        this.get_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "name").into(),
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
    if let Some(this) = this {
        this.get_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "numFrames").into(),
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
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Scene instance initializer>", mc),
        Method::from_builtin(class_init, "<Scene class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("labels", Some(labels), None),
        ("name", Some(name), None),
        ("numFrames", Some(num_frames), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PRIVATE_INSTANCE_SLOTS: &[(&str, &str, &str, &str)] = &[
        (NS_RUFFLE_INTERNAL, "name", "", "String"),
        (NS_RUFFLE_INTERNAL, "labels", "", "Array"),
        (NS_RUFFLE_INTERNAL, "numFrames", "", "int"),
    ];
    write.define_private_slot_instance_traits(PRIVATE_INSTANCE_SLOTS);

    class
}
