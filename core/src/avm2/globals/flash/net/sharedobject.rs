//! `flash.net.SharedObject` builtin/prototype

use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::TObject;
use crate::avm2::traits::Trait;
use crate::avm2::{Activation, Error, Namespace, Object, QName, Value};
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let data = activation
            .context
            .avm2
            .classes()
            .object
            .construct(activation, &[])?;
        this.set_property(
            &QName::new(Namespace::public(), "data").into(),
            data.into(),
            activation,
        )?;
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

fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("SharedObject.getLocal not implemented");
    let class = activation.context.avm2.classes().sharedobject;
    let new_shared_object = class.construct(activation, &[])?;

    Ok(new_shared_object.into())
}

fn flush<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("SharedObject.flush not implemented");
    Ok(Value::Undefined)
}

/// Construct `SharedObject`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.net"), "SharedObject"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init, "<SharedObject instance initializer>", mc),
        Method::from_builtin(class_init, "<SharedObject class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::public(), "data"),
        QName::new(Namespace::public(), "Object").into(),
        None,
    ));

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("getLocal", get_local)];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("flush", flush)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);
    class
}
