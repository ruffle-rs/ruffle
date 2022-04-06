//! Object representation for events

use crate::avm2::activation::Activation;
use crate::avm2::events::{Event, EventData};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Event objects.
pub fn event_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(EventObject(GcCell::allocate(
        activation.context.gc_context,
        EventObjectData {
            base,
            event: Event::new("", EventData::Empty),
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
        event: Event<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = match event.event_data() {
            EventData::Empty => activation.avm2().classes().event,
            EventData::FullScreen { .. } => activation.avm2().classes().fullscreenevent,
            EventData::Mouse { .. } => activation.avm2().classes().mouseevent,
            EventData::IOError { .. } => activation.avm2().classes().ioerrorevent,
        };

        let proto = class.prototype();
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut event_object: Object<'gc> = EventObject(GcCell::allocate(
            activation.context.gc_context,
            EventObjectData { base, event },
        ))
        .into();
        event_object.install_instance_slots(activation);

        //TODO: Find a way to call the constructor's default initializer
        //without overwriting the event we just put on the object.

        Ok(event_object)
    }
}

impl<'gc> TObject<'gc> for EventObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object((*self).into()))
    }

    fn as_event(&self) -> Option<Ref<Event<'gc>>> {
        Some(Ref::map(self.0.read(), |d| &d.event))
    }

    fn as_event_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<Event<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.event))
    }
}
