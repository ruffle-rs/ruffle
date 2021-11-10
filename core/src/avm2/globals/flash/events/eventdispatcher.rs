//! `flash.events.EventDispatcher` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::{
    dispatch_event as dispatch_event_internal, parent_of, NS_EVENT_DISPATCHER,
};
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{DispatchObject, Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use gc_arena::{GcCell, MutationContext};

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
            &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target").into(),
            target,
            activation,
        )?;
        this.init_property(
            this,
            &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list").into(),
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
    if let Some(this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list").into(),
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

        Avm2::register_broadcast_listener(&mut activation.context, this, event_type);
    }

    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.removeEventListener`.
pub fn remove_event_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list").into(),
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
    if let Some(this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list").into(),
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

/// Implements `EventDispatcher.willTrigger`.
pub fn will_trigger<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let dispatch_list = this
            .get_property(
                this,
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "dispatch_list").into(),
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
                &QName::new(Namespace::private(NS_EVENT_DISPATCHER), "target").into(),
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

    if let Some(this) = this {
        Ok(dispatch_event_internal(activation, this, event)?.into())
    } else {
        Ok(false.into())
    }
}

/// Implements `flash.events.EventDispatcher`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `EventDispatcher.toString`.
///
/// This is an undocumented function, but MX will VerifyError if this isn't
/// present.
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let object_proto = activation.avm2().prototypes().object;
    object_proto
        .get_property(
            object_proto,
            &QName::dynamic_name("toString").into(),
            activation,
        )?
        .coerce_to_object(activation)?
        .call(this, args, activation)
}

/// Construct `EventDispatcher`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "EventDispatcher"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<EventDispatcher instance initializer>", mc),
        Method::from_builtin(class_init, "<EventDispatcher class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    write.implements(QName::new(Namespace::package("flash.events"), "IEventDispatcher").into());

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("addEventListener", add_event_listener),
        ("removeEventListener", remove_event_listener),
        ("hasEventListener", has_event_listener),
        ("willTrigger", will_trigger),
        ("dispatchEvent", dispatch_event),
        ("toString", to_string),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

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
