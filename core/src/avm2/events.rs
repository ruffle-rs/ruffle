//! Core event structure

use crate::avm2::Avm2;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_events_event_dispatcher as slots;
use crate::avm2::object::{EventObject, FunctionObject, FunctionObjectWeak, Object, TObject as _};
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use fnv::FnvHashMap;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
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
}

impl<'gc> Event<'gc> {
    /// Construct a new event of a given type.
    pub fn new(event_type: AvmString<'gc>) -> Self {
        Event {
            bubbles: false,
            cancelable: false,
            cancelled: false,
            propagation: PropagationMode::Allow,
            current_target: None,
            event_phase: EventPhase::AtTarget,
            target: None,
            event_type,
        }
    }

    pub fn event_type(&self) -> AvmString<'gc> {
        self.event_type
    }

    pub fn set_event_type(&mut self, event_type: AvmString<'gc>) {
        self.event_type = event_type;
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
    fn get_event(&self, event: AvmString<'gc>) -> Option<&BTreeMap<i32, Vec<EventHandler<'gc>>>> {
        self.0.get(&event)
    }

    /// Get all of the event handlers for a given event type, for mutation.
    ///
    /// If the event type does not exist, it will be added to the dispatch
    /// list.
    fn get_event_mut(
        &mut self,
        event: AvmString<'gc>,
    ) -> &mut BTreeMap<i32, Vec<EventHandler<'gc>>> {
        self.0.entry(event).or_default()
    }

    /// Get a single priority level of event handlers for a given event type,
    /// for mutation.
    fn get_event_priority_mut(
        &mut self,
        event: AvmString<'gc>,
        priority: i32,
    ) -> &mut Vec<EventHandler<'gc>> {
        self.0
            .entry(event)
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
    ///
    /// When `use_weak_reference` is true, the dispatch list holds a weak
    /// reference to the handler — letting it (and anything captured by it)
    /// be collected when the only references are weak. Matches Flash's
    /// `addEventListener(..., useWeakReference=true)` behavior.
    ///
    /// Before adding the new handler, dead weak entries for the same event
    /// name are pruned. The other prune path inside `iter_event_handlers`
    /// fires only when the dispatcher actually dispatches that event;
    /// long-lived event sources whose subscribers register but rarely
    /// receive the event would otherwise accumulate one dead weak entry
    /// per discarded listener indefinitely. The add-time prune cost is
    /// O(N) over the existing entries for the same event, trivial
    /// relative to the unbounded growth it prevents.
    pub fn add_event_listener(
        &mut self,
        mc: &Mutation<'gc>,
        event: AvmString<'gc>,
        priority: i32,
        handler: FunctionObject<'gc>,
        use_capture: bool,
        use_weak_reference: bool,
    ) {
        // Lazy weak cleanup for this event name. See doc above.
        if let Some(event_sheaf) = self.0.get_mut(&event) {
            for set in event_sheaf.values_mut() {
                set.retain(|eh| eh.handler.upgrade(mc).is_some());
            }
        }

        let new_handler = EventHandler::new(handler, use_capture, use_weak_reference);

        if let Some(event_sheaf) = self.get_event(event) {
            for other_set in event_sheaf.values() {
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
    /// removed from any priority in the list. Matches against both strong and
    /// weak registrations of the same handler (pointer identity).
    pub fn remove_event_listener(
        &mut self,
        event: AvmString<'gc>,
        handler: FunctionObject<'gc>,
        use_capture: bool,
    ) {
        // `use_weak_reference` is irrelevant for matching: `HandlerRef::as_ptr`
        // yields the same identity for Strong and Weak of the same allocation.
        let old_handler = EventHandler::new(handler, use_capture, false);

        for set in self.get_event_mut(event).values_mut() {
            if let Some(pos) = set.iter().position(|h| *h == old_handler) {
                set.remove(pos);
            }
        }
    }

    /// Determine if there are any event listeners in this dispatch list.
    pub fn has_event_listener(&self, event: AvmString<'gc>) -> bool {
        if let Some(event_sheaf) = self.get_event(event) {
            for set in event_sheaf.values() {
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
    ///
    /// Weak handlers whose target has already been garbage collected are
    /// transparently pruned from the dispatch list as a side effect.
    pub fn iter_event_handlers<'a>(
        &'a mut self,
        mc: &'a Mutation<'gc>,
        event: AvmString<'gc>,
        use_capture: bool,
    ) -> impl 'a + Iterator<Item = FunctionObject<'gc>> {
        // Prune dead weak listeners before iterating. Keeps DispatchList from
        // growing unbounded when Flex (or other code) registers many weak
        // listeners over the lifetime of a long-lived event source.
        let bucket = self.get_event_mut(event);
        for set in bucket.values_mut() {
            set.retain(|eh| eh.handler.upgrade(mc).is_some());
        }

        bucket
            .iter()
            .rev()
            .flat_map(|(_p, v)| v.iter())
            .filter(move |eh| eh.use_capture == use_capture)
            .filter_map(move |eh| eh.handler.upgrade(mc))
    }
}

impl Default for DispatchList<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference to an event handler, either strong (default) or weak
/// (when `useWeakReference=true` is passed to `addEventListener`).
///
/// Weak listener references are required by frameworks whose binding system
/// subscribes short-lived observers to long-lived event sources. The Flex
/// SDK's `mx.binding.PropertyWatcher`, for example, registers every
/// `propertyChange` listener as weak so the watcher (and its enclosing
/// document) is not retained by every singleton it subscribes to. Treating
/// those references as strong creates an unbounded retention chain across
/// the lifetime of those singletons.
#[derive(Clone, Collect)]
#[collect(no_drop)]
enum HandlerRef<'gc> {
    Strong(FunctionObject<'gc>),
    Weak(FunctionObjectWeak<'gc>),
}

impl<'gc> HandlerRef<'gc> {
    /// Stable pointer identity for the underlying allocation. Both Strong and
    /// Weak variants for the same function yield the same value, so a listener
    /// registered weak can still be removed via `removeEventListener` (which
    /// always calls in with a strong FunctionObject).
    fn as_ptr(&self) -> *const () {
        match self {
            HandlerRef::Strong(f) => Gc::as_ptr(f.0).cast(),
            HandlerRef::Weak(w) => GcWeak::as_ptr(w.0).cast(),
        }
    }

    /// Upgrade to a callable FunctionObject. Returns None if the listener was
    /// weak and the underlying object has been collected.
    fn upgrade(&self, mc: &Mutation<'gc>) -> Option<FunctionObject<'gc>> {
        match self {
            HandlerRef::Strong(f) => Some(*f),
            HandlerRef::Weak(w) => w.0.upgrade(mc).map(FunctionObject),
        }
    }
}

/// A single instance of an event handler.
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct EventHandler<'gc> {
    /// The event handler to call.
    handler: HandlerRef<'gc>,

    /// Indicates if this handler should only be called for capturing events
    /// (when `true`), or if it should only be called for bubbling and
    /// at-target events (when `false`).
    use_capture: bool,
}

impl<'gc> EventHandler<'gc> {
    fn new(handler: FunctionObject<'gc>, use_capture: bool, use_weak_reference: bool) -> Self {
        let handler = if use_weak_reference {
            HandlerRef::Weak(FunctionObjectWeak(Gc::downgrade(handler.0)))
        } else {
            HandlerRef::Strong(handler)
        };
        Self {
            handler,
            use_capture,
        }
    }
}

impl PartialEq for EventHandler<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.use_capture == rhs.use_capture
            && std::ptr::eq(self.handler.as_ptr(), rhs.handler.as_ptr())
    }
}

impl Eq for EventHandler<'_> {}

impl Hash for EventHandler<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.use_capture.hash(state);
        (self.handler.as_ptr() as usize).hash(state);
    }
}

/// Retrieve the parent of a given `EventDispatcher`.
///
/// `EventDispatcher` does not provide a generic way for it's subclasses to
/// indicate ancestry. Instead, only specific event targets provide a hierarchy
/// to traverse. If no hierarchy is available, this returns `None`, as if the
/// target had no parent.
pub fn parent_of(target: Object<'_>) -> Option<Object<'_>> {
    if let Some(dobj) = target.as_display_object()
        && let Some(dparent) = dobj.parent()
        && let Some(parent) = dparent.object2()
    {
        return Some(parent.into());
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
    real_target: Object<'gc>,
    current_target: Object<'gc>,
    event: EventObject<'gc>,
    simulate_dispatch: bool,
) {
    avm_debug!(
        activation.context.avm2,
        "Event dispatch: {} to {current_target:?}",
        event.event().event_type(),
    );

    let dispatch_list = dispatcher.get_slot(slots::DISPATCH_LIST).as_object();

    if dispatch_list.is_none() {
        // Objects with no dispatch list act as if they had an empty one
        return;
    }

    let dispatch_list = dispatch_list.unwrap();

    let mut evtmut = event.event_mut(activation.gc());
    let name = evtmut.event_type();
    let use_capture = evtmut.phase() == EventPhase::Capturing;

    let mc = activation.gc();
    let handlers: Vec<FunctionObject<'gc>> = dispatch_list
        .as_dispatch_mut(mc)
        .expect("Internal dispatch list is missing during dispatch!")
        .iter_event_handlers(mc, name, use_capture)
        .collect();

    if !handlers.is_empty() {
        evtmut.set_target(real_target);
        evtmut.set_current_target(current_target);
    }

    drop(evtmut);

    if simulate_dispatch {
        return;
    }

    for handler in handlers.iter() {
        if event.event().is_propagation_stopped_immediately() {
            break;
        }

        let global = activation.context.avm2.toplevel_global_object().unwrap();

        let args = &[event.into()];
        let result = handler.call(activation, global.into(), FunctionArgs::from_slice(args));
        if let Err(err) = result {
            let event_name = event.event().event_type();

            Avm2::uncaught_error(
                activation,
                None, // TODO we need to set this, but how?
                err,
                &format!("Error dispatching event \"{}\"", event_name),
            );
        }
    }
}

pub fn dispatch_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    event: EventObject<'gc>,
    simulate_dispatch: bool,
) -> bool {
    let target = this.get_slot(slots::TARGET).as_object().unwrap_or(this);

    let mut ancestor_list = Vec::new();
    // Edge case - during button construction, we fire bubbling events for objects
    // that are in the hierarchy (and have `DisplayObject.stage` return the actual stage),
    // but do not yet have their *parent* object constructed. As a result, we walk through
    // the parent DisplayObject hierarchy, only adding ancestors that have objects constructed.
    let mut parent = target.as_display_object().and_then(|dobj| dobj.parent());
    while let Some(parent_dobj) = parent {
        if let Some(parent_obj) = parent_dobj.object2() {
            ancestor_list.push(parent_obj.into());
        }
        parent = parent_dobj.parent();
    }

    event
        .event_mut(activation.gc())
        .set_phase(EventPhase::Capturing);

    for ancestor in ancestor_list.iter().rev() {
        if event.event().is_propagation_stopped() {
            break;
        }

        dispatch_event_to_target(
            activation,
            *ancestor,
            target,
            *ancestor,
            event,
            simulate_dispatch,
        );
    }

    event
        .event_mut(activation.gc())
        .set_phase(EventPhase::AtTarget);

    if !event.event().is_propagation_stopped() {
        dispatch_event_to_target(activation, this, target, target, event, simulate_dispatch);
    }

    event
        .event_mut(activation.context.gc_context)
        .set_phase(EventPhase::Bubbling);

    if event.event().is_bubbling() {
        for ancestor in ancestor_list.iter() {
            if event.event().is_propagation_stopped() {
                break;
            }

            dispatch_event_to_target(
                activation,
                *ancestor,
                target,
                *ancestor,
                event,
                simulate_dispatch,
            );
        }
    }

    // If the target is set, the event was handled
    event.event().target.is_some()
}

/// Like `dispatch_event`, but does not run the Capturing and Bubbling phases,
/// and dispatches the event regardless of whether propagation has been stopped.
/// This matches FP's event broadcasting logic.
pub fn broadcast_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    event: EventObject<'gc>,
) {
    let target = this.get_slot(slots::TARGET).as_object().unwrap_or(this);

    event
        .event_mut(activation.gc())
        .set_phase(EventPhase::AtTarget);

    dispatch_event_to_target(activation, this, target, target, event, false);
}
