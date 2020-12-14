//! Core event structure

use crate::avm2::object::Object;
use crate::avm2::string::AvmString;
use gc_arena::Collect;

/// Which phase of event dispatch is currently occurring.
#[derive(Copy, Clone, Collect, Debug, PartialEq, Eq)]
#[collect(require_static)]
pub enum EventPhase {
    /// The event has yet to be fired on the target and is descending the
    /// ancestors of the event target.
    Capturing,

    /// The event is currently firing on the target.
    AtTarget,

    /// The event has already fired on the target and is ascending the
    /// ancestors of the event target.
    Bubbling,
}

impl Into<u32> for EventPhase {
    fn into(self) -> u32 {
        match self {
            Self::Capturing => 0,
            Self::AtTarget => 1,
            Self::Bubbling => 2,
        }
    }
}

/// How this event is allowed to propagate.
#[derive(Copy, Clone, Collect, Debug, PartialEq, Eq)]
#[collect(require_static)]
pub enum PropagationMode {
    /// Propagate events normally.
    AllowPropagation,

    /// Stop capturing or bubbling events.
    StopPropagation,

    /// Stop running event handlers altogether.
    StopImmediatePropagation,
}

/// Represents data fields of an event that can be fired on an object that
/// implements `IEventDispatcher`.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct Event<'gc> {
    /// Whether or not the event "bubbles" - fires on it's parents after it
    /// fires on the child.
    bubbles: bool,

    /// Whether or not the event has a default response that an event handler
    /// can request to not occur.
    cancelable: bool,

    /// Whether or not the event's default response has been cancelled.
    cancelled: bool,

    /// Whether or not event propagation has stopped.
    propagation: PropagationMode,

    /// The object currently having it's event handlers invoked.
    current_target: Option<Object<'gc>>,

    /// The current event phase.
    event_phase: EventPhase,

    /// The object this event was dispatched on.
    target: Option<Object<'gc>>,

    /// The name of the event being triggered.
    event_type: AvmString<'gc>,
}

impl<'gc> Event<'gc> {
    /// Construct a new event of a given type.
    pub fn new<S>(event_type: S) -> Self
    where
        S: Into<AvmString<'gc>>,
    {
        Event {
            bubbles: false,
            cancelable: false,
            cancelled: false,
            propagation: PropagationMode::AllowPropagation,
            current_target: None,
            event_phase: EventPhase::Bubbling,
            target: None,
            event_type: event_type.into(),
        }
    }

    pub fn event_type(&self) -> AvmString<'gc> {
        self.event_type
    }

    pub fn set_event_type<S>(&mut self, event_type: S)
    where
        S: Into<AvmString<'gc>>,
    {
        self.event_type = event_type.into();
    }

    pub fn is_bubbling(&self) -> bool {
        self.bubbles
    }

    pub fn set_bubbles(&mut self, bubbling: bool) {
        self.bubbles = bubbling;
    }

    pub fn is_cancelable(&self) -> bool {
        self.cancelable
    }

    pub fn set_cancelable(&mut self, cancelable: bool) {
        self.cancelable = cancelable;
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn cancel(&mut self) {
        if self.cancelable {
            self.cancelled = true;
        }
    }

    pub fn is_propagation_stopped(&self) -> bool {
        self.propagation != PropagationMode::AllowPropagation
    }

    pub fn stop_propagation(&mut self) {
        if self.propagation != PropagationMode::StopImmediatePropagation {
            self.propagation = PropagationMode::StopPropagation;
        }
    }

    pub fn is_propagation_stopped_immediately(&self) -> bool {
        self.propagation == PropagationMode::StopImmediatePropagation
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.propagation = PropagationMode::StopImmediatePropagation;
    }

    pub fn phase(&self) -> EventPhase {
        self.event_phase
    }

    pub fn target(&self) -> Option<Object<'gc>> {
        self.target
    }

    pub fn current_target(&self) -> Option<Object<'gc>> {
        self.current_target
    }
}
