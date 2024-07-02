//! Object representation for events

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::events::Event;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ByteArrayObject, ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;
use crate::display_object::{DisplayObject, InteractiveObject, TInteractiveObject};
use crate::events::KeyCode;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

/// A class instance allocator that allocates Event objects.
pub fn event_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(EventObject(GcCell::new(
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
pub struct EventObject<'gc>(pub GcCell<'gc, EventObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct EventObjectWeak<'gc>(pub GcWeakCell<'gc, EventObjectData<'gc>>);

#[derive(Clone, Collect)]
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
    pub fn bare_default_event<S>(context: &mut UpdateContext<'_, 'gc>, event_type: S) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        Self::bare_event(context, event_type, false, false)
    }

    /// Create a bare Event instance while skipping the usual `construct()` pipeline.
    /// It's just slightly faster and doesn't require an Activation.
    /// Note that if you need an Event subclass, you need to construct it via .construct().
    pub fn bare_event<S>(
        context: &mut UpdateContext<'_, 'gc>,
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

        let event_object = EventObject(GcCell::new(
            context.gc_context,
            EventObjectData { base, event },
        ));

        // not needed, as base Event has no instance slots.
        // yes, this is flimsy. Could call this if install_instance_slots only took gc_context.
        // event_object.install_instance_slots(activation);

        event_object.into()
    }

    pub fn mouse_event<S>(
        activation: &mut Activation<'_, 'gc>,
        event_type: S,
        target: DisplayObject<'gc>,
        related_object: Option<InteractiveObject<'gc>>,
        delta: i32,
        bubbles: bool,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let local = target.local_mouse_position(&activation.context);

        let event_type: AvmString<'gc> = event_type.into();

        let mouse_event_cls = activation.avm2().classes().mouseevent;
        mouse_event_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    // bubbles
                    bubbles.into(),
                    // cancellable
                    false.into(),
                    // localX
                    local.x.to_pixels().into(),
                    // localY
                    local.y.to_pixels().into(),
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

    pub fn text_event<S>(
        activation: &mut Activation<'_, 'gc>,
        event_type: S,
        text: AvmString<'gc>,
        bubbles: bool,
        cancelable: bool,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let event_type: AvmString<'gc> = event_type.into();

        let text_event_cls = activation.avm2().classes().textevent;
        text_event_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    // bubbles
                    bubbles.into(),
                    // cancelable
                    cancelable.into(),
                    // text
                    text.into(),
                ],
            )
            .unwrap() // we don't expect to break here
    }

    pub fn sample_data_event<S>(
        activation: &mut Activation<'_, 'gc>,
        event_type: S,
        position: u32,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let event_type: AvmString<'gc> = event_type.into();

        let storage = ByteArrayStorage::new();
        let data = ByteArrayObject::from_storage(activation, storage).unwrap();

        let sample_data_event_cls = activation.avm2().classes().sampledataevent;
        sample_data_event_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    //bubbles
                    false.into(),
                    //cancelable
                    false.into(),
                    position.into(),
                    data.into(),
                ],
            )
            .unwrap() // we don't expect to break here
    }

    pub fn net_status_event<S>(
        activation: &mut Activation<'_, 'gc>,
        event_type: S,
        info: Vec<(impl Into<AvmString<'gc>>, impl Into<AvmString<'gc>>)>,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let info_object = activation
            .avm2()
            .classes()
            .object
            .construct(activation, &[])
            .unwrap();
        for (key, value) in info {
            info_object
                .set_public_property(key.into(), Value::String(value.into()), activation)
                .unwrap();
        }

        let event_type: AvmString<'gc> = event_type.into();

        let net_status_cls = activation.avm2().classes().netstatusevent;
        net_status_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    //bubbles
                    false.into(),
                    //cancelable
                    false.into(),
                    info_object.into(),
                ],
            )
            .unwrap() // we don't expect to break here
    }

    pub fn progress_event<S>(
        activation: &mut Activation<'_, 'gc>,
        event_type: S,
        bytes_loaded: u64,
        bytes_total: u64,
        bubbles: bool,
        cancelable: bool,
    ) -> Object<'gc>
    where
        S: Into<AvmString<'gc>>,
    {
        let event_type: AvmString<'gc> = event_type.into();

        let progress_event_cls = activation.avm2().classes().progressevent;
        progress_event_cls
            .construct(
                activation,
                &[
                    event_type.into(),
                    // bubbles
                    bubbles.into(),
                    // cancelable
                    cancelable.into(),
                    // bytesLoaded
                    (bytes_loaded as f64).into(),
                    // bytesToal
                    (bytes_total as f64).into(),
                ],
            )
            .unwrap() // we don't expect to break here
    }
}

impl<'gc> TObject<'gc> for EventObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_event(&self) -> Option<Ref<Event<'gc>>> {
        Some(Ref::map(self.0.read(), |d| &d.event))
    }

    fn as_event_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<Event<'gc>>> {
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
