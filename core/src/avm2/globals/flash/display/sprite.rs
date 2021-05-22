//! `flash.display.Sprite` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, StageObject, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.Sprite`'s instance constructor.
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

/// Implements `flash.display.Sprite`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `graphics`.
pub fn graphics<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(dobj) = this.as_display_object() {
            // Lazily initialize the `Graphics` object in a hidden property.
            let graphics = match this.get_property(
                this,
                &QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
                activation,
            )? {
                Value::Undefined | Value::Null => {
                    let mut graphics_proto = activation.context.avm2.prototypes().graphics;
                    let graphics_constr = graphics_proto
                        .get_property(
                            graphics_proto,
                            &QName::new(Namespace::public(), "constructor"),
                            activation,
                        )
                        .and_then(|v| v.coerce_to_object(activation))
                        .expect("Video proto needs constr");

                    let graphics = Value::from(StageObject::for_display_object(
                        activation.context.gc_context,
                        dobj,
                        graphics_constr,
                        graphics_proto,
                    ));
                    this.set_property(
                        this,
                        &QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
                        graphics.clone(),
                        activation,
                    )?;
                    graphics
                }
                graphics => graphics,
            };
            return Ok(graphics);
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Sprite`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Sprite"),
        Some(
            QName::new(
                Namespace::package("flash.display"),
                "DisplayObjectContainer",
            )
            .into(),
        ),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(&str, Option<NativeMethod>, Option<NativeMethod>)] =
        &[("graphics", Some(graphics), None)];
    write.define_public_builtin_instance_properties(PUBLIC_INSTANCE_PROPERTIES);

    // Slot for lazy-initialized Graphics object.
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
        QName::new(Namespace::package("flash.display"), "Graphics").into(),
        None,
    ));

    class
}
