//! `flash.events.Event` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{EventObject, Object, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
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

/// Implements `target` property's getter
pub fn target<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.target().map(|o| o.into()).unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `currentTarget` property's getter
pub fn current_target<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt
            .current_target()
            .map(|o| o.into())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `eventPhase` property's getter
pub fn event_phase<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        let event_phase: u32 = evt.phase().into();
        return Ok(event_phase.into());
    }

    Ok(Value::Undefined)
}

/// Implements `clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        let evt_proto = activation.avm2().system_prototypes.as_ref().unwrap().event;

        return Ok(EventObject::from_event(
            activation.context.gc_context,
            Some(evt_proto),
            evt.clone(),
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// Implements `formatToString`
pub fn format_to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let class_name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let mut stringified_params = Vec::new();

        if let Some(params) = args.get(1..) {
            for param_name in params {
                let param_name = QName::dynamic_name(match param_name {
                    Value::Undefined | Value::Null => "null".into(),
                    _ => param_name.coerce_to_string(activation)?,
                });

                let param_value = this
                    .get_property(this, &param_name, activation)?
                    .coerce_to_debug_string(activation)?;
                stringified_params.push(format!(" {}={}", param_name.local_name(), param_value));
            }
        }

        return Ok(AvmString::new(
            activation.context.gc_context,
            format!("[{}{}]", class_name, stringified_params.join("")),
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// Implements `isDefaultPrevented`
pub fn is_default_prevented<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_cancelled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `preventDefault`
pub fn prevent_default<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.cancel();
    }

    Ok(Value::Undefined)
}

/// Implements `stopPropagation`
pub fn stop_propagation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.stop_propagation();
    }

    Ok(Value::Undefined)
}

/// Implements `stopImmediatePropagation`
pub fn stop_immediate_propagation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.stop_immediate_propagation();
    }

    Ok(Value::Undefined)
}

/// Implements `toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        return this.value_of(activation.context.gc_context);
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
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "target"),
        Method::from_builtin(target),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "currentTarget"),
        Method::from_builtin(current_target),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "eventPhase"),
        Method::from_builtin(event_phase),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "clone"),
        Method::from_builtin(clone),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "formatToString"),
        Method::from_builtin(format_to_string),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "isDefaultPrevented"),
        Method::from_builtin(is_default_prevented),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "preventDefault"),
        Method::from_builtin(prevent_default),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "stopPropagation"),
        Method::from_builtin(stop_propagation),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "stopImmediatePropagation"),
        Method::from_builtin(stop_immediate_propagation),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "toString"),
        Method::from_builtin(to_string),
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
