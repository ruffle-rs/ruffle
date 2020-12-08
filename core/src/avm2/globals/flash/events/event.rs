//! `flash.events.Event` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{EventObject, Object, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.Event`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.set_event_type(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)?,
        );
        evt.set_bubbles(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Bool(false))
                .coerce_to_boolean(),
        );
        evt.set_cancelable(
            args.get(2)
                .cloned()
                .unwrap_or(Value::Bool(false))
                .coerce_to_boolean(),
        );
    }

    Ok(Value::Undefined)
}

/// Implements `flash.events.Event`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `bubbles` property's getter
pub fn bubbles<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_bubbling().into());
    }

    Ok(Value::Undefined)
}

/// Implements `cancelable` property's getter
pub fn cancelable<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_cancelable().into());
    }

    Ok(Value::Undefined)
}

/// Implements `type` property's getter
pub fn get_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.event_type().into());
    }

    Ok(Value::Undefined)
}

/// Construct `Event`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "Event"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "bubbles"),
        Method::from_builtin(bubbles),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "cancelable"),
        Method::from_builtin(cancelable),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "type"),
        Method::from_builtin(get_type),
    ));

    class
}

/// Object deriver for `Event`
pub fn event_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    Ok(EventObject::derive(
        base_proto,
        activation.context.gc_context,
        class,
        scope,
    ))
}
