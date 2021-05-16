//! Object representation for events

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::events::Event;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance deriver that constructs Event objects.
pub fn event_deriver<'gc>(
    mut constr: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    let base_proto = constr
        .get_property(
            constr,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)?;

    Ok(EventObject::derive(
        base_proto,
        activation.context.gc_context,
        class,
        scope,
    ))
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct EventObject<'gc>(GcCell<'gc, EventObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct EventObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The event this object holds.
    event: Event<'gc>,
}

impl<'gc> EventObject<'gc> {
    /// Convert a bare event into it's object representation.
    pub fn from_event(
        mc: MutationContext<'gc, '_>,
        base_proto: Option<Object<'gc>>,
        event: Event<'gc>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(base_proto, ScriptObjectClass::NoClass);

        EventObject(GcCell::allocate(mc, EventObjectData { base, event })).into()
    }

    /// Instantiate an event subclass.
    pub fn derive(
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        EventObject(GcCell::allocate(
            mc,
            EventObjectData {
                base,
                event: Event::new(""),
            },
        ))
        .into()
    }
}

impl<'gc> TObject<'gc> for EventObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::EventObject(*self);
        Ok(EventObject::from_event(
            activation.context.gc_context,
            Some(this),
            Event::new(""),
        ))
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::EventObject(*self);

        Ok(Self::derive(
            this,
            activation.context.gc_context,
            class,
            scope,
        ))
    }

    fn value_of(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let read = self.0.read();
        let event_type = read.event.event_type();
        let bubbles = read.event.is_bubbling();
        let cancelable = read.event.is_cancelable();
        let phase = read.event.phase() as u32;

        Ok(AvmString::new(
            mc,
            format!(
                "[Event type=\"{}\" bubbles={} cancelable={} eventPhase={}]",
                event_type, bubbles, cancelable, phase
            ),
        )
        .into())
    }

    fn as_event(&self) -> Option<Ref<Event<'gc>>> {
        Some(Ref::map(self.0.read(), |d| &d.event))
    }

    fn as_event_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<Event<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.event))
    }
}
