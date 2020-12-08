//! `flash.events.Event` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{EventObject, Object, TObject};
use crate::avm2::scope::Scope;
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

/// Construct `Event`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    Class::new(
        QName::new(Namespace::package("flash.events"), "Event"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    )
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
