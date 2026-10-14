//! Timer handling for `setInterval`/`setTimeout`/`Timer` AVM timers.
//!
//! We tick the timers during our normal frame loop for deterministic operation.
//! Every update runs all timers that are due, each at most once, in the order they
//! were created, like Flash Player does when it checks its timers.

mod timestamp;

use crate::avm1::ExecutionReason;
use crate::avm1::{Activation, ActivationIdentifier, Object as Avm1Object, Value as Avm1Value};
use crate::avm2::error::make_null_or_undefined_error;
use crate::avm2::object::FunctionObject as Avm2FunctionObject;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Error as Avm2Error, FunctionArgs, Value as Avm2Value,
};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::AvmString;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::duration::FloatDuration;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::RangeTo;
use std::time::Duration;
use timestamp::TimerTimestamp;

/// The key of an active timer, which orders timers by the time when they should
/// fire and their sequence numbers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Collect)]
#[collect(require_static)]
struct TimerKey {
    tick_time: TimerTimestamp,

    /// Sequence number of the timer.
    ///
    /// Sequence numbers have to be strictly increasing and never reused.
    /// They are used to order timers that should fire at the same time.
    sequence: u64,
}

impl TimerKey {
    /// The range of keys of timers that should have fired before the given
    /// time.
    fn due_before(time: TimerTimestamp) -> RangeTo<Self> {
        ..Self {
            tick_time: time,
            sequence: 0,
        }
    }
}

/// Manages the collection of timers.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Timers<'gc> {
    /// The active timers keyed by the time when they should fire.
    timers: BTreeMap<TimerKey, Timer<'gc>>,

    /// The timers created by `setInterval`/`setTimeout`, keyed by their IDs.
    ids: BTreeMap<i32, Timer<'gc>>,

    /// The last ID assigned to a timer created by `setInterval`/`setTimeout`.
    last_id: i32,

    /// The last sequence number assigned to a timer.
    last_sequence: u64,

    /// The current global time.
    #[collect(require_static)]
    current_time: TimerTimestamp,
}

impl<'gc> Timers<'gc> {
    /// Ticks all timers and runs necessary callbacks.
    pub fn update_timers(context: &mut UpdateContext<'gc>, dt: FloatDuration) -> Option<f64> {
        context.timers.current_time += dt.to_std();

        if context.timers.is_empty() {
            return None;
        }

        let level0 = context.stage.root_clip();

        // Run every timer that is due once, in creation order, even when an interval
        // missed several ticks. This is what Flash Player does when it checks its timers.
        let current_time = context.timers.current_time;
        let mut due: Vec<TimerKey> = context
            .timers
            .timers
            .range(TimerKey::due_before(current_time))
            .map(|(&key, _)| key)
            .collect();

        // Sequence numbers are unique, so unstable is fine.
        due.sort_unstable_by_key(|key| key.sequence);

        for key in due {
            // We have to be careful because the timer list can be mutated while updating;
            // a timer callback could add more timers, clear timers (including this one),
            // change their delays, etc.
            let Some(&timer) = context.timers.timers.get(&key) else {
                continue;
            };

            // TODO: Can we avoid these clones?
            let callback = timer.0.callback.clone();

            let cancel_timer = match callback {
                TimerCallback::Avm1Function { func, params } => {
                    if let Some(level0) = level0 {
                        let mut avm1_activation = Activation::from_nothing(
                            context,
                            ActivationIdentifier::root("[Timer Callback]"),
                            level0,
                        );
                        let result = func.call(
                            "[Timer Callback]",
                            &mut avm1_activation,
                            Avm1Value::Undefined,
                            &params,
                        );

                        if let Err(e) = result {
                            tracing::error!("Unhandled AVM1 error in timer callback: {}", e);
                        }
                    } else {
                        tracing::warn!("Skipping AVM1 timer as there's no root");
                    }

                    false
                }
                TimerCallback::Avm1Method {
                    this,
                    method_name,
                    params,
                } => {
                    // If you add a timer onto a MovieClip and then remove the clip
                    // The timer should stop firing and be canceled
                    // Because we store an object and not a value, we have to check for a clip like this

                    let mut removed = false;

                    // We can't use as_display_object + as_movie_clip here as we explicitly don't want to convert `super`s
                    if let Some(DisplayObject::MovieClip(mc)) = this.as_display_object_no_super() {
                        // Note that we don't want to fire the timer here
                        if mc.avm1_removed() {
                            removed = true;
                        }
                    }

                    if !removed {
                        if let Some(level0) = level0 {
                            let mut avm1_activation = Activation::from_nothing(
                                context,
                                ActivationIdentifier::root("[Timer Callback]"),
                                level0,
                            );
                            let result = this.call_method(
                                method_name,
                                &params,
                                &mut avm1_activation,
                                ExecutionReason::Special,
                            );

                            if let Err(e) = result {
                                tracing::error!("Unhandled AVM1 error in timer callback: {}", e);
                            }
                        } else {
                            tracing::warn!("Skipping AVM1 timer as there's no root");
                        }

                        false
                    } else {
                        true
                    }
                }
                TimerCallback::Avm2Callback { closure, params } => {
                    let domain = context.avm2.stage_domain();
                    let mut avm2_activation = Avm2Activation::from_domain(context, domain);

                    let mut run_closure = || -> Result<Avm2Value<'gc>, Avm2Error<'gc>> {
                        // Reproduce FP's behavior
                        let Some(closure) = closure else {
                            return Err(make_null_or_undefined_error(
                                &mut avm2_activation,
                                Avm2Value::Null,
                                None,
                            ));
                        };

                        let params = FunctionArgs::from_slice(&params);
                        closure.call(&mut avm2_activation, Avm2Value::Null, params)
                    };

                    match run_closure() {
                        Ok(v) => v.coerce_to_boolean(),
                        Err(err) => {
                            Avm2::uncaught_error(
                                &mut avm2_activation,
                                None, // TODO do we need to set this?
                                err,
                                "Error in AVM2 timer callback",
                            );
                            false
                        }
                    }
                }
            };

            crate::player::Player::run_actions(context);

            // The callback could have removed the timer.
            let timers = &mut context.timers;
            if !timers.contains(timer) {
                continue;
            }

            if timer.0.is_timeout || cancel_timer {
                // Timeouts only fire once, and a false return value from the callback cancels intervals.
                timers.remove_timer(timer);
            } else if timer.key() == key {
                // Reset setInterval timers, unless the callback changed the delay, which
                // already rescheduled it. Missed ticks are skipped, like in Flash Player.
                timers.timers.remove(&key);
                let interval = timer.0.interval.get();
                let next_tick_time = key.tick_time + interval;
                let tick_time = if next_tick_time < current_time {
                    current_time + interval
                } else {
                    next_tick_time
                };
                let key = TimerKey { tick_time, ..key };
                timer.0.key.set(key);
                timers.timers.insert(key, timer);
            }
        }

        // Return estimated time until next timer tick.
        context.timers.timers.first_key_value().map(|(key, _)| {
            FloatDuration::from_std(key.tick_time - context.timers.current_time).as_millis()
        })
    }

    /// The minimum interval we allow for timers.
    const MIN_INTERVAL: i32 = 10;

    /// Creates a new `Timers` collection.
    pub fn new() -> Self {
        Self {
            timers: Default::default(),
            ids: Default::default(),
            last_id: 0,
            last_sequence: 0,
            current_time: TimerTimestamp::ZERO,
        }
    }

    /// Whether there are no timers active.
    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    /// Whether the given timer is active.
    pub fn contains(&self, timer: Timer<'gc>) -> bool {
        self.timers.contains_key(&timer.key())
    }

    /// Registers a new timer for `setInterval`/`setTimeout` and returns the timer ID.
    pub fn add_timer_with_id(
        &mut self,
        mc: &Mutation<'gc>,
        callback: TimerCallback<'gc>,
        interval: i32,
        is_timeout: bool,
    ) -> i32 {
        // The ID has to be incremented first, because Flash Player never assigns timer ID 0,
        // and neither should we, as some content uses it as an initial "not yet set up" value.
        self.last_id = self.last_id.wrapping_add(1);
        let id = self.last_id;
        self.add_timer(mc, Some(id), callback, interval, is_timeout);
        id
    }

    /// Registers a new timer without a ID and returns it.
    ///
    /// Flash Player does not use up IDs for `flash.utils.Timer`.
    pub fn add_timer_without_id(
        &mut self,
        mc: &Mutation<'gc>,
        callback: TimerCallback<'gc>,
        interval: i32,
        is_timeout: bool,
    ) -> Timer<'gc> {
        self.add_timer(mc, None, callback, interval, is_timeout)
    }

    fn add_timer(
        &mut self,
        mc: &Mutation<'gc>,
        id: Option<i32>,
        callback: TimerCallback<'gc>,
        interval: i32,
        is_timeout: bool,
    ) -> Timer<'gc> {
        // SANITY: Set a minimum interval so we don't spam too much.
        let interval = Duration::from_millis(interval.max(Self::MIN_INTERVAL) as u64);

        let timer = Timer(Gc::new(
            mc,
            TimerData {
                id,
                callback,
                key: Cell::new(TimerKey {
                    tick_time: self.current_time + interval,
                    sequence: 0,
                }),
                interval: Cell::new(interval),
                is_timeout,
            },
        ));
        self.insert(timer);
        timer
    }

    /// Makes a timer active, ordering it after all other timers.
    fn insert(&mut self, timer: Timer<'gc>) {
        self.last_sequence += 1;
        let key = TimerKey {
            sequence: self.last_sequence,
            ..timer.key()
        };
        timer.0.key.set(key);
        self.timers.insert(key, timer);
        if let Some(id) = timer.0.id {
            self.ids.insert(id, timer);
        }
    }

    /// Removes a timer.
    pub fn remove_timer(&mut self, timer: Timer<'gc>) -> bool {
        if self.timers.remove(&timer.key()).is_none() {
            return false;
        }
        if let Some(id) = timer.0.id {
            self.ids.remove(&id);
        }
        true
    }

    /// Removes a timer created by `setInterval`/`setTimeout`.
    pub fn remove_timer_by_id(&mut self, id: i32) -> bool {
        match self.ids.get(&id) {
            Some(&timer) => self.remove_timer(timer),
            None => false,
        }
    }

    pub fn remove_all(&mut self) {
        self.timers.clear();
        self.ids.clear();
    }

    /// Changes the delay of a timer.
    pub fn set_delay(&mut self, timer: Timer<'gc>, interval: i32) {
        // SANITY: Set a minimum interval so we don't spam too much.
        let interval = Duration::from_millis(interval.max(Self::MIN_INTERVAL) as u64);

        if !self.remove_timer(timer) {
            panic!("Changing delay of non-existent timer");
        }

        timer.0.interval.set(interval);
        timer.0.key.set(TimerKey {
            tick_time: self.current_time + interval,
            ..timer.key()
        });
        // Flash Player runs a timer with a changed delay after the timers created
        // before the change, so it is ordered as if it was created again.
        self.insert(timer);
    }
}

impl Default for Timers<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A timer created via `setInterval`/`setTimeout`/`Timer`.
/// Runs a callback when it ticks.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Timer<'gc>(Gc<'gc, TimerData<'gc>>);

impl Timer<'_> {
    /// The key of the timer in `Timers`.
    fn key(self) -> TimerKey {
        self.0.key.get()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
struct TimerData<'gc> {
    /// The ID of the timer, if it was created by `setInterval`/`setTimeout`.
    id: Option<i32>,

    /// The callback that this timer runs when it fires.
    ///
    /// A callback is either a function object, or a parent object with a method name.
    callback: TimerCallback<'gc>,

    /// The key of the timer in `Timers`, with the time when this timer should fire.
    ///
    /// Its sequence number is assigned when the timer is created and again when its
    /// delay changes. The key must not change while the timer is active.
    key: Cell<TimerKey>,

    /// The interval between timer ticks.
    interval: Cell<Duration>,

    /// This timer only fires once if `is_timeout` is true.
    is_timeout: bool,
}

/// A callback fired by a `setInterval`/`setTimeout` timer.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum TimerCallback<'gc> {
    Avm1Function {
        func: Avm1Object<'gc>,
        /// The parameters to pass to the callback function.
        params: Vec<Avm1Value<'gc>>,
    },

    Avm1Method {
        this: Avm1Object<'gc>,
        method_name: AvmString<'gc>,
        params: Vec<Avm1Value<'gc>>,
    },

    Avm2Callback {
        closure: Option<Avm2FunctionObject<'gc>>,
        params: Vec<Avm2Value<'gc>>,
    },
}
