//! `flash.events.EventDispatcher` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::EventPhase;
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{DispatchObject, Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};

const NS_EVENT_DISPATCHER: &str = "https://ruffle.rs/AS3/impl/EventDispatcher/";

/// Implements `flash.events.EventDispatcher`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let target = args.get(0).cloned().unwrap_or(Value::Null);
        let dispatch_list = DispatchObject::empty_list(activation.context.gc_context);

        this.init_property(
            this,
            &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target"),
            target,
            activation,
        )?;
        this.init_property(
            this,
            &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
            dispatch_list.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.addEventListener`.
pub fn add_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let event_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let listener = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let use_capture = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Bool(false))
            .coerce_to_boolean();
        let priority = args
            .get(3)
            .cloned()
            .unwrap_or(Value::Integer(0))
            .coerce_to_i32(activation)?;

        //TODO: If we ever get weak GC references, we should respect `useWeakReference`.
        dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .add_event_listener(event_type, priority, listener, use_capture);
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.removeEventListener`.
pub fn remove_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let event_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let listener = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let use_capture = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Bool(false))
            .coerce_to_boolean();

        dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .remove_event_listener(event_type, listener, use_capture);
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.hasEventListener`.
pub fn has_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let event_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        return Ok(dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .has_event_listener(event_type)
            .into());
    }

    Ok(Value::Undefined)
}

/// Retrieve the parent of a given `EventDispatcher`.
///
/// `EventDispatcher` does not provide a generic way for it's subclasses to
/// indicate ancestry. Instead, only specific event targets provide a hierarchy
/// to traverse. If no hierarchy is available, this returns `None`, as if the
/// target had no parent.
fn parent_of(target: Object<'_>) -> Option<Object<'_>> {
    if let Some(dobj) = target.as_display_object() {
        if let Some(dparent) = dobj.parent() {
            if let Value::Object(parent) = dparent.object2() {
                return Some(parent);
            }
        }
    }

    None
}

/// Implements `EventDispatcher.willTrigger`.
pub fn will_trigger<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let event_type = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if dispatch_list
            .as_dispatch_mut(activation.context.gc_context)
            .ok_or_else(|| Error::from("Internal properties should have what I put in them"))?
            .has_event_listener(event_type)
        {
            return Ok(true.into());
        }

        let target = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target"),
                activation,
            )?
            .coerce_to_object(activation)
            .ok()
            .unwrap_or(this);

        if let Some(parent) = parent_of(target) {
            return will_trigger(activation, Some(parent), args);
        }
    }

    Ok(false.into())
}

/// Call all of the event handlers on a given target.
///
/// The `target` is the current target of the `event`. `event` must be a valid
/// `EventObject`, or this function will panic. You must have already set the
/// event's phase to match what targets you are dispatching to, or you will
/// call the wrong handlers.
pub fn dispatch_event_to_target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    mut target: Object<'gc>,
    event: Object<'gc>,
) -> Result<(), Error> {
    let dispatch_list = target
        .get_property(
            target,
            &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
            activation,
        )?
        .coerce_to_object(activation)?;

    let mut evtmut = event.as_event_mut(activation.context.gc_context).unwrap();
    let name = evtmut.event_type();
    let use_capture = evtmut.phase() == EventPhase::Capturing;

    evtmut.set_current_target(target);

    drop(evtmut);

    let handlers: Vec<Object<'gc>> = dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .ok_or_else(|| Error::from("Internal dispatch list is missing during dispatch!"))?
        .iter_event_handlers(name, use_capture)
        .collect();

    for handler in handlers.iter() {
        if event
            .as_event()
            .unwrap()
            .is_propagation_stopped_immediately()
        {
            break;
        }

        handler.call(
            activation.global_scope().coerce_to_object(activation).ok(),
            &[event.into()],
            activation,
            None,
        )?;
    }

    Ok(())
}

/// Implements `EventDispatcher.dispatchEvent`.
pub fn dispatch_event<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let event = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?;

    if event.as_event().is_none() {
        return Err("Dispatched Events must be subclasses of Event.".into());
    }

    if let Some(mut this) = this {
        let target = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target"),
                activation,
            )?
            .coerce_to_object(activation)
            .ok()
            .unwrap_or(this);

        let mut ancestor_list = Vec::new();
        let mut parent = parent_of(target);
        while let Some(par) = parent {
            ancestor_list.push(par);
            parent = parent_of(par);
        }

        let mut evtmut = event.as_event_mut(activation.context.gc_context).unwrap();

        evtmut.set_phase(EventPhase::Capturing);
        evtmut.set_target(target);

        drop(evtmut);

        for ancestor in ancestor_list.iter().rev() {
            if event.as_event().unwrap().is_propagation_stopped() {
                break;
            }

            dispatch_event_to_target(activation, *ancestor, event)?;
        }

        event
            .as_event_mut(activation.context.gc_context)
            .unwrap()
            .set_phase(EventPhase::AtTarget);

        if !event.as_event().unwrap().is_propagation_stopped() {
            dispatch_event_to_target(activation, target, event)?;
        }

        event
            .as_event_mut(activation.context.gc_context)
            .unwrap()
            .set_phase(EventPhase::Bubbling);

        if event.as_event().unwrap().is_bubbling() {
            for ancestor in ancestor_list.iter() {
                if event.as_event().unwrap().is_propagation_stopped() {
                    break;
                }

                dispatch_event_to_target(activation, *ancestor, event)?;
            }
        }
    }

    let was_not_cancelled = !event.as_event().unwrap().is_cancelled();

    Ok(was_not_cancelled.into())
}

/// Implements `flash.events.EventDispatcher`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `EventDispatcher`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "EventDispatcher"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    write.implements(QName::new(Namespace::package("flash.events"), "IEventDispatcher").into());

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "addEventListener"),
        Method::from_builtin(add_event_listener),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "removeEventListener"),
        Method::from_builtin(remove_event_listener),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "hasEventListener"),
        Method::from_builtin(has_event_listener),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "willTrigger"),
        Method::from_builtin(will_trigger),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "dispatchEvent"),
        Method::from_builtin(dispatch_event),
    ));

    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target"),
        QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "BareObject").into(),
        None,
    ));
    write.define_instance_trait(Trait::from_slot(
        QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list"),
        QName::new(Namespace::private(NS_RUFFLE_INTERNAL), "BareObject").into(),
        None,
    ));

    class
}
