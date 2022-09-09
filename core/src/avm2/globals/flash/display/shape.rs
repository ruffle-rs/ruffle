//! `flash.display.Shape` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, StageObject, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::display_object::Graphic;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.Shape`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let new_do = Graphic::new_with_avm2(&mut activation.context, this);

            this.init_display_object(activation.context.gc_context, new_do.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Shape`'s class constructor.
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
                &Multiname::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
                activation,
            )? {
                Value::Undefined | Value::Null => {
                    let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                    this.set_property(
                        &Multiname::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
                        graphics,
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

/// Construct `Shape`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Shape"),
        Some(Multiname::new(
            Namespace::package("flash.display"),
            "DisplayObject",
        )),
        Method::from_builtin(instance_init, "<Shape instance initializer>", mc),
        Method::from_builtin(class_init, "<Shape class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("graphics", Some(graphics), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    // Slot for lazy-initialized Graphics object.
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "graphics"),
        Multiname::new(Namespace::package("flash.display"), "Graphics"),
        None,
    ));

    class
}
