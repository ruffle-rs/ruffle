//! Object representation for events

use crate::avm2::activation::Activation;
use crate::avm2::events::Event;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;
use crate::display_object::{DisplayObject, InteractiveObject, TInteractiveObject};
use crate::events::KeyCode;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

/// A class instance allocator that allocates Event objects.
pub fn event_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    Ok(EventObject(GcCell::allocate(
        activation.context.gc_context,
        EventObjectData {
            base,
            event: Event::new(""),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
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
    /// Create a bare Event instance while skipping the usual `construct()` pipeline.
    /// It's just slightly faster and doesn't require an Activation.
    /// This is equivalent to
    /// classes.event.construct(activation, &[event_type, false, false])
    pub fn bare_default_event<S>(
        context: &mut UpdateContext<'_, 'gc, '_>,
        event_type: S,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        Self::bare_event(context, event_type, false, false)
    }

    /// Create a bare Event instance while skipping the usual `construct()` pipeline.
    /// It's just slightly faster and doesn't require an Activation.
    /// Note that if you need an Event subclass, you need to construct it via .construct().
    pub fn bare_event<S>(
        context: &mut UpdateContext<'_, 'gc, '_>,
        event_type: S,
        bubbles: bool,
        cancelable: bool,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let class = context.avm2.classes().event;
        let base = ScriptObjectData::new(class);

        let mut event = Event::new(event_type);
        event.set_bubbles(bubbles);
        event.set_cancelable(cancelable);

        let event_object = EventObject(GcCell::allocate(
            context.gc_context,
            EventObjectData { base, event },
        ));

        // not needed, as base Event has no instance slots.
        // yes, this is flimsy. Could call this if install_instance_slots only took gc_context.
        // event_object.install_instance_slots(activation);

        event_object.into()
    }

    pub fn mouse_event<S>(
        activation: &mut Activation<'_, 'gc, '_>,
        event_type: S,
        target: DisplayObject<'gc>,
        related_object: Option<InteractiveObject<'gc>>,
        delta: i32,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let local_pos = target.global_to_local(*activation.context.mouse_position);

        let event_type: AvmString<'gc> = event_type.into();

        let mouse_event_cls = activation.avm2().classes().mouseevent;
        mouse_event_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    // bubbles
                    true.into(),
                    // cancellable
                    false.into(),
                    // localX
                    local_pos.0.to_pixels().into(),
                    // localY
                    local_pos.1.to_pixels().into(),
                    // relatedObject
                    related_object
                        .map(|o| o.as_displayobject().object2())
                        .unwrap_or(Value::Null),
                    // ctrlKey
                    activation
                        .context
                        .input
                        .is_key_down(KeyCode::Control)
                        .into(),
                    // altKey
                    activation.context.input.is_key_down(KeyCode::Alt).into(),
                    // shiftKey
                    activation.context.input.is_key_down(KeyCode::Shift).into(),
                    // buttonDown
                    activation.context.input.is_mouse_down().into(),
                    // delta
                    delta.into(),
                ],
            )
            .unwrap() // we don't expect to break here
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

impl<'gc> Debug for EventObject<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0.try_read() {
            Ok(obj) => f
                .debug_struct("EventObject")
                .field("type", &obj.event.event_type())
                .field("class", &obj.base.debug_class_name())
                .field("ptr", &self.0.as_ptr())
                .finish(),
            Err(err) => f
                .debug_struct("EventObject")
                .field("type", &err)
                .field("class", &err)
                .field("ptr", &self.0.as_ptr())
                .finish(),
        }
    }
}
