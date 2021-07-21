//! Object representation for events

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::events::Event;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Event objects.
pub fn event_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(EventObject(GcCell::allocate(
        activation.context.gc_context,
        EventObjectData {
            base,
            event: Event::new(""),
        },
    ))
    .into())
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
    ///
    /// This function supports constructing subclasses of `Event`; as a result,
    /// we will pull the `prototype` off the `class` given to us.
    pub fn from_event(
        activation: &mut Activation<'_, 'gc, '_>,
        class: Object<'gc>,
        event: Event<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let proto = class
            .get_property(
                class,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut event_object: Object<'gc> = EventObject(GcCell::allocate(
            activation.context.gc_context,
            EventObjectData { base, event },
        ))
        .into();
        event_object.install_instance_traits(activation, class)?;

        //TODO: Find a way to call the constructor's default initializer
        //without overwriting the event we just put on the object.

        Ok(event_object)
    }
}

impl<'gc> TObject<'gc> for EventObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(EventObject(GcCell::allocate(
            activation.context.gc_context,
            EventObjectData {
                base,
                event: Event::new(""),
            },
        ))
        .into())
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
