//! Core event structure

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2007;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use fnv::FnvHashMap;
use gc_arena::Collect;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Which phase of event dispatch is currently occurring.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EventPhase {
    /// The event has yet to be fired on the target and is descending the
    /// ancestors of the event target.
    Capturing = 1,

    /// The event is currently firing on the target.
    AtTarget = 2,

    /// The event has already fired on the target and is ascending the
    /// ancestors of the event target.
    Bubbling = 3,
}

/// How this event is allowed to propagate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PropagationMode {
    /// Propagate events normally.
    Allow,

    /// Stop capturing or bubbling events.
    Stop,

    /// Stop running event handlers altogether.
    StopImmediate,
}

/// Represents data fields of an event that can be fired on an object that
/// implements `IEventDispatcher`.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Event<'gc> {
    /// Whether the event "bubbles" - fires on its parents after it
    /// fires on the child.
    bubbles: bool,

    /// Whether the event has a default response that an event handler
    /// can request to not occur.
    cancelable: bool,

    /// Whether the event's default response has been cancelled.
    cancelled: bool,

    /// Whether event propagation has stopped.
    #[collect(require_static)]
    propagation: PropagationMode,

    /// The object currently having its event handlers invoked.
    current_target: Option<Object<'gc>>,

    /// The current event phase.
    #[collect(require_static)]
    event_phase: EventPhase,

    /// The object this event was dispatched on.
    target: Option<Object<'gc>>,

    /// The name of the event being triggered.
    event_type: AvmString<'gc>,

    /// Whether is event has been dispatched before.
    dispatched: bool,
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
            propagation: PropagationMode::Allow,
            current_target: None,
            event_phase: EventPhase::AtTarget,
            target: None,
            event_type: event_type.into(),
            dispatched: false,
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
        self.propagation != PropagationMode::Allow
    }

    pub fn stop_propagation(&mut self) {
        if self.propagation != PropagationMode::StopImmediate {
            self.propagation = PropagationMode::Stop;
        }
    }

    pub fn is_propagation_stopped_immediately(&self) -> bool {
        self.propagation == PropagationMode::StopImmediate
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.propagation = PropagationMode::StopImmediate;
    }

    pub fn phase(&self) -> EventPhase {
        self.event_phase
    }

    pub fn set_phase(&mut self, phase: EventPhase) {
        self.event_phase = phase;
    }

    pub fn target(&self) -> Option<Object<'gc>> {
        self.target
    }

    pub fn set_target(&mut self, target: Object<'gc>) {
        self.target = Some(target)
    }

    pub fn current_target(&self) -> Option<Object<'gc>> {
        self.current_target
    }

    pub fn set_current_target(&mut self, current_target: Object<'gc>) {
        self.current_target = Some(current_target)
    }
}

/// A set of handlers organized by event type, priority, and order added.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct DispatchList<'gc>(FnvHashMap<AvmString<'gc>, BTreeMap<i32, Vec<EventHandler<'gc>>>>);

impl<'gc> DispatchList<'gc> {
    /// Construct a new dispatch list.
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Get all of the event handlers for a given event type, if such a type
    /// exists.
    fn get_event(
        &self,
        event: impl Into<AvmString<'gc>>,
    ) -> Option<&BTreeMap<i32, Vec<EventHandler<'gc>>>> {
        self.0.get(&event.into())
    }

    /// Get all of the event handlers for a given event type, for mutation.
    ///
    /// If the event type does not exist, it will be added to the dispatch
    /// list.
    fn get_event_mut(
        &mut self,
        event: impl Into<AvmString<'gc>>,
    ) -> &mut BTreeMap<i32, Vec<EventHandler<'gc>>> {
        self.0.entry(event.into()).or_default()
    }

    /// Get a single priority level of event handlers for a given event type,
    /// for mutation.
    fn get_event_priority_mut(
        &mut self,
        event: impl Into<AvmString<'gc>>,
        priority: i32,
    ) -> &mut Vec<EventHandler<'gc>> {
        self.0
            .entry(event.into())
            .or_default()
            .entry(priority)
            .or_default()
    }

    /// Add an event handler to this dispatch list.
    ///
    /// This enforces the invariant that an `EventHandler` must not appear at
    /// more than one priority (since we can't enforce that with clever-er data
    /// structure selection). If an event handler already exists, it will not
    /// be added again, and this function will silently fail.
    pub fn add_event_listener(
        &mut self,
        event: impl Into<AvmString<'gc>> + Clone,
        priority: i32,
        handler: Object<'gc>,
        use_capture: bool,
    ) {
        let new_handler = EventHandler::new(handler, use_capture);

        if let Some(event_sheaf) = self.get_event(event.clone()) {
            for (_other_prio, other_set) in event_sheaf.iter() {
                if other_set.contains(&new_handler) {
                    return;
                }
            }
        }

        self.get_event_priority_mut(event, priority)
            .push(new_handler);
    }

    /// Remove an event handler from this dispatch list.
    ///
    /// Any listener that has the same handler and capture-phase flag will be
    /// removed from any priority in the list.
    pub fn remove_event_listener(
        &mut self,
        event: impl Into<AvmString<'gc>>,
        handler: Object<'gc>,
        use_capture: bool,
    ) {
        let old_handler = EventHandler::new(handler, use_capture);

        for (_prio, set) in self.get_event_mut(event).iter_mut() {
            if let Some(pos) = set.iter().position(|h| *h == old_handler) {
                set.remove(pos);
            }
        }
    }

    /// Determine if there are any event listeners in this dispatch list.
    pub fn has_event_listener(&self, event: impl Into<AvmString<'gc>>) -> bool {
        if let Some(event_sheaf) = self.get_event(event) {
            for (_prio, set) in event_sheaf.iter() {
                if !set.is_empty() {
                    return true;
                }
            }
        }

        false
    }

    /// Yield the event handlers on this dispatch list for a given event.
    ///
    /// Event handlers will be yielded in the order they are intended to be
    /// executed.
    ///
    /// `use_capture` indicates if you want handlers that execute during the
    /// capture phase, or handlers that execute during the bubble and target
    /// phases.
    pub fn iter_event_handlers<'a>(
        &'a mut self,
        event: impl Into<AvmString<'gc>>,
        use_capture: bool,
    ) -> impl 'a + Iterator<Item = Object<'gc>> {
        self.get_event_mut(event)
            .iter()
            .rev()
            .flat_map(|(_p, v)| v.iter())
            .filter(move |eh| eh.use_capture == use_capture)
            .map(|eh| eh.handler)
    }
}

impl Default for DispatchList<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A single instance of an event handler.
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct EventHandler<'gc> {
    /// The event handler to call.
    handler: Object<'gc>,

    /// Indicates if this handler should only be called for capturing events
    /// (when `true`), or if it should only be called for bubbling and
    /// at-target events (when `false`).
    use_capture: bool,
}

impl<'gc> EventHandler<'gc> {
    fn new(handler: Object<'gc>, use_capture: bool) -> Self {
        Self {
            handler,
            use_capture,
        }
    }
}

impl PartialEq for EventHandler<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.use_capture == rhs.use_capture && Object::ptr_eq(self.handler, rhs.handler)
    }
}

impl Eq for EventHandler<'_> {}

impl Hash for EventHandler<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.use_capture.hash(state);
        self.handler.as_ptr().hash(state);
    }
}

/// Retrieve the parent of a given `EventDispatcher`.
///
/// `EventDispatcher` does not provide a generic way for it's subclasses to
/// indicate ancestry. Instead, only specific event targets provide a hierarchy
/// to traverse. If no hierarchy is available, this returns `None`, as if the
/// target had no parent.
pub fn parent_of(target: Object<'_>) -> Option<Object<'_>> {
    if let Some(dobj) = target.as_display_object() {
        if let Some(dparent) = dobj.parent() {
            if let Value::Object(parent) = dparent.object2() {
                return Some(parent);
            }
        }
    }

    None
}

/// Call all of the event handlers on a given target.
///
/// The `target` is the current target of the `event`. `event` must be a valid
/// `EventObject`, or this function will panic. You must have already set the
/// event's phase to match what targets you are dispatching to, or you will
/// call the wrong handlers.
fn dispatch_event_to_target<'gc>(
    activation: &mut Activation<'_, 'gc>,
    dispatcher: Object<'gc>,
    target: Object<'gc>,
    event: Object<'gc>,
    simulate_dispatch: bool,
) -> Result<(), Error<'gc>> {
    avm_debug!(
        activation.context.avm2,
        "Event dispatch: {} to {target:?}",
        event.as_event().unwrap().event_type(),
    );

    let internal_ns = activation.avm2().namespaces.flash_events_internal;
    let dispatch_list = dispatcher
        .get_property(&Multiname::new(internal_ns, "_dispatchList"), activation)?
        .as_object();

    if dispatch_list.is_none() {
        // Objects with no dispatch list act as if they had an empty one
        return Ok(());
    }

    let dispatch_list = dispatch_list.unwrap();

    let mut evtmut = event.as_event_mut(activation.context.gc_context).unwrap();
    let name = evtmut.event_type();
    let use_capture = evtmut.phase() == EventPhase::Capturing;

    let handlers: Vec<Object<'gc>> = dispatch_list
        .as_dispatch_mut(activation.context.gc_context)
        .ok_or_else(|| Error::from("Internal dispatch list is missing during dispatch!"))?
        .iter_event_handlers(name, use_capture)
        .collect();

    evtmut.set_current_target(target);

    if !handlers.is_empty() {
        evtmut.dispatched = true;
    }

    drop(evtmut);

    if simulate_dispatch {
        return Ok(());
    }

    for handler in handlers.iter() {
        if event
            .as_event()
            .unwrap()
            .is_propagation_stopped_immediately()
        {
            break;
        }

        let global = activation.context.avm2.toplevel_global_object().unwrap();

        if let Err(err) = handler.call(global.into(), &[event.into()], activation) {
            tracing::error!(
                "Error dispatching event {:?} to handler {:?} : {:?}",
                event,
                handler,
                err,
            );
        }
    }

    Ok(())
}

pub fn dispatch_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    event: Object<'gc>,
    simulate_dispatch: bool,
) -> Result<bool, Error<'gc>> {
    let internal_ns = activation.avm2().namespaces.flash_events_internal;
    let target = this
        .get_property(&Multiname::new(internal_ns, "_target"), activation)?
        .as_object()
        .unwrap_or(this);

    let mut ancestor_list = Vec::new();
    // Edge case - during button construction, we fire bubbling events for objects
    // that are in the hierarchy (and have `DisplayObject.stage` return the actual stage),
    // but do not yet have their *parent* object constructed. As a result, we walk through
    // the parent DisplayObject hierarchy, only adding ancestors that have objects constructed.
    let mut parent = target.as_display_object().and_then(|dobj| dobj.parent());
    while let Some(parent_dobj) = parent {
        if let Value::Object(parent_obj) = parent_dobj.object2() {
            ancestor_list.push(parent_obj);
        }
        parent = parent_dobj.parent();
    }

    let dispatched = event.as_event().unwrap().dispatched;

    let event = if dispatched {
        event
            .call_public_property("clone", &[], activation)?
            .as_object()
            .ok_or_else(|| make_error_2007(activation, "event"))?
    } else {
        event
    };

    let mut evtmut = event.as_event_mut(activation.context.gc_context).unwrap();

    evtmut.set_phase(EventPhase::Capturing);
    evtmut.set_target(target);

    drop(evtmut);

    for ancestor in ancestor_list.iter().rev() {
        if event.as_event().unwrap().is_propagation_stopped() {
            break;
        }

        dispatch_event_to_target(activation, *ancestor, *ancestor, event, simulate_dispatch)?;
    }

    event
        .as_event_mut(activation.context.gc_context)
        .unwrap()
        .set_phase(EventPhase::AtTarget);

    if !event.as_event().unwrap().is_propagation_stopped() {
        dispatch_event_to_target(activation, this, target, event, simulate_dispatch)?;
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

            dispatch_event_to_target(activation, *ancestor, *ancestor, event, simulate_dispatch)?;
        }
    }

    let handled = event.as_event().unwrap().dispatched;
    Ok(handled)
}
